# Algorithm Specifications

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0  
**Date:** October 26, 2025

---

## 1. Input Type Detection

### Algorithm

```rust
fn identify_input(input: &str) -> InputType {
    // Priority order matters!
    if is_ip(input) { return IP; }
    if is_asn(input) { return ASN; }
    if is_asn_id(input) { return ASNID; }
    if is_domain(input) { return Domain; }
    return Organization;  // fallback
}
```

### Implementation Details

**IP Detection:**
- Use stdlib `IpAddr::from_str()`
- Handles IPv4 and IPv6
- Complexity: O(n) where n = string length
- Fast path for common formats

**ASN Detection:**
```rust
fn is_asn(input: &str) -> bool {
    input.len() > 2 
        && input[..2].eq_ignore_ascii_case("AS")
        && input[2..].chars().all(|c| c.is_ascii_digit())
}
```
- Complexity: O(n)
- No allocations
- ASCII-only for performance

**Domain Detection:**
- Compiled regex (once at startup)
- Pattern: `^(?i)[a-z0-9-_]+(\.[a-z0-9-]+)+\.?$`
- Complexity: O(n) with regex engine
- Consider DNS label length limits (63 chars/label, 253 total)

**Performance:**
- Target: <1μs per input (cached regex)
- Zero allocations for valid inputs
- Batch mode: use `rayon` for parallelization

---

## 2. DNS Resolution Strategy

### Concurrent Resolution Algorithm

```rust
async fn resolve_batch(domains: Vec<&str>) -> Vec<Result<Vec<IpAddr>>> {
    // Step 1: Check cache (parallel)
    let (cached, uncached): (Vec<_>, Vec<_>) = domains
        .into_iter()
        .partition(|d| cache.contains_key(d));
    
    // Step 2: Concurrent resolution of uncached
    let tasks: FuturesUnordered<_> = uncached
        .into_iter()
        .map(|domain| async move {
            // Query A and AAAA concurrently
            let (ipv4, ipv6) = tokio::join!(
                resolver.lookup_a(domain),
                resolver.lookup_aaaa(domain)
            );
            merge_results(ipv4, ipv6)
        })
        .collect();
    
    // Step 3: Await all with timeout
    timeout(Duration::from_secs(10), tasks.collect()).await
}
```

### Retry Strategy

**Exponential Backoff:**
- Initial delay: 100ms
- Max delay: 5s
- Jitter: ±20%
- Max attempts: 3

```rust
let backoff = ExponentialBackoff {
    initial_interval: Duration::from_millis(100),
    max_interval: Duration::from_secs(5),
    max_elapsed_time: Some(Duration::from_secs(30)),
    multiplier: 2.0,
    randomization_factor: 0.2,
    ..Default::default()
};
```

### Caching Strategy

**Cache Key:** `dns:{domain}`  
**TTL:** Use DNS record TTL (min 60s, max 24h)  
**Eviction:** LRU when memory limit reached  
**Negative Caching:** Cache NXDOMAIN for 5 minutes

**Performance:**
- Cache hit: <1ms
- Cache miss + resolution: <50ms P99
- Concurrent limit: 1000 tasks

---

## 3. CIDR Conversion

### Range to CIDR Algorithm

Convert IP range [start, end] to minimal set of CIDR blocks.

```rust
fn range_to_cidrs(start: IpAddr, end: IpAddr) -> Vec<IpNet> {
    let mut cidrs = Vec::new();
    let mut current = start;
    
    while current <= end {
        // Find largest prefix that:
        // 1. Starts at current
        // 2. Doesn't exceed end
        let prefix_len = find_max_prefix(current, end);
        let cidr = IpNet::new(current, prefix_len)?;
        cidrs.push(cidr);
        
        // Move to next block
        current = next_ip(cidr.broadcast());
    }
    
    cidrs
}

fn find_max_prefix(start: IpAddr, end: IpAddr) -> u8 {
    let start_u128 = ip_to_u128(start);
    let end_u128 = ip_to_u128(end);
    let range_size = end_u128 - start_u128 + 1;
    
    // Find largest power of 2 that fits
    let prefix_len = (128 - range_size.trailing_zeros()) as u8;
    
    // Ensure alignment
    let aligned_prefix = find_alignment(start, prefix_len);
    aligned_prefix.max(prefix_len)
}
```

**Complexity:** O(log n) where n = range size  
**Example:**
- Input: `192.168.0.0` - `192.168.1.255`
- Output: `[192.168.0.0/23]`

---

## 4. CIDR Aggregation

### Merge Adjacent CIDRs

```rust
fn aggregate(mut cidrs: Vec<IpNet>) -> Vec<IpNet> {
    if cidrs.is_empty() { return vec![]; }
    
    // Step 1: Sort by network address
    cidrs.sort_by_key(|c| (c.network(), c.prefix_len()));
    
    // Step 2: Merge adjacent and overlapping
    let mut result = vec![cidrs[0]];
    
    for cidr in cidrs.into_iter().skip(1) {
        let last = result.last_mut().unwrap();
        
        if can_merge(last, &cidr) {
            *last = merge(last, &cidr);
        } else if !last.contains(&cidr.network()) {
            // Not contained, add as new
            result.push(cidr);
        }
        // else: contained, skip
    }
    
    result
}

fn can_merge(a: &IpNet, b: &IpNet) -> bool {
    // Check if adjacent and same prefix length
    a.prefix_len() == b.prefix_len() 
        && next_ip(a.broadcast()) == b.network()
        && a.prefix_len() > 0
}

fn merge(a: &IpNet, b: &IpNet) -> IpNet {
    // Create supernet with prefix_len - 1
    IpNet::new(a.network(), a.prefix_len() - 1).unwrap()
}
```

**Complexity:** O(n log n) for sort, O(n) for merge = O(n log n)  
**Example:**
- Input: `[192.168.0.0/24, 192.168.1.0/24]`
- Output: `[192.168.0.0/23]`

---

## 5. Overlap Detection

### Interval Tree Approach

```rust
struct IntervalTree {
    root: Option<Box<Node>>,
}

struct Node {
    interval: IpNet,
    max_end: IpAddr,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl IntervalTree {
    // O(n log n) construction
    fn build(cidrs: Vec<IpNet>) -> Self {
        let mut sorted = cidrs;
        sorted.sort_by_key(|c| c.network());
        Self { root: Self::build_rec(&sorted) }
    }
    
    // O(log n + k) where k = overlaps found
    fn find_overlaps(&self, target: &IpNet) -> Vec<IpNet> {
        let mut result = Vec::new();
        self.find_overlaps_rec(&self.root, target, &mut result);
        result
    }
    
    fn find_overlaps_rec(
        &self, 
        node: &Option<Box<Node>>, 
        target: &IpNet, 
        result: &mut Vec<IpNet>
    ) {
        if let Some(n) = node {
            // Check current node
            if overlaps(&n.interval, target) {
                result.push(n.interval);
            }
            
            // Prune left subtree if no overlap possible
            if let Some(ref left) = n.left {
                if left.max_end >= target.network() {
                    self.find_overlaps_rec(&n.left, target, result);
                }
            }
            
            // Always check right
            self.find_overlaps_rec(&n.right, target, result);
        }
    }
}

fn overlaps(a: &IpNet, b: &IpNet) -> bool {
    a.contains(&b.network()) 
        || a.contains(&b.broadcast())
        || b.contains(&a.network())
}
```

**Complexity:**
- Construction: O(n log n)
- Query: O(log n + k) where k = matches
- Space: O(n)

---

## 6. Cache Eviction Policy

### LRU (Least Recently Used)

```rust
struct LruCache<K, V> {
    map: HashMap<K, (V, *mut Node<K>)>,
    list: DoublyLinkedList<K>,
    capacity: usize,
}

impl<K, V> LruCache<K, V> {
    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, node_ptr)) = self.map.get(key) {
            // Move to front (most recent)
            self.list.move_to_front(*node_ptr);
            Some(value)
        } else {
            None
        }
    }
    
    fn insert(&mut self, key: K, value: V) {
        if self.map.len() >= self.capacity {
            // Evict least recently used (back of list)
            if let Some(evicted_key) = self.list.pop_back() {
                self.map.remove(&evicted_key);
            }
        }
        
        let node_ptr = self.list.push_front(key.clone());
        self.map.insert(key, (value, node_ptr));
    }
}
```

**Complexity:**
- Get: O(1)
- Insert: O(1)
- Space: O(capacity)

### TTL-Based Eviction

```rust
struct TtlCache<K, V> {
    map: HashMap<K, CacheEntry<V>>,
    expiry_queue: BinaryHeap<ExpiryEntry<K>>,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K, V> TtlCache<K, V> {
    fn get(&mut self, key: &K) -> Option<&V> {
        self.evict_expired();  // Lazy eviction
        
        self.map.get(key)
            .filter(|entry| entry.expires_at > Instant::now())
            .map(|entry| &entry.value)
    }
    
    fn evict_expired(&mut self) {
        let now = Instant::now();
        while let Some(entry) = self.expiry_queue.peek() {
            if entry.expires_at <= now {
                self.expiry_queue.pop();
                self.map.remove(&entry.key);
            } else {
                break;
            }
        }
    }
}
```

---

## 7. Rate Limiting Algorithm

### Token Bucket

```rust
struct TokenBucket {
    tokens: AtomicU64,
    capacity: u64,
    refill_rate: u64,  // tokens per second
    last_refill: Atomic<Instant>,
}

impl TokenBucket {
    async fn acquire(&self, n: u64) -> Result<()> {
        loop {
            self.refill();
            
            let current = self.tokens.load(Ordering::Relaxed);
            if current >= n {
                if self.tokens.compare_exchange(
                    current, 
                    current - n,
                    Ordering::Release,
                    Ordering::Relaxed
                ).is_ok() {
                    return Ok(());
                }
            } else {
                // Wait for refill
                let wait_time = self.calculate_wait(n);
                tokio::time::sleep(wait_time).await;
            }
        }
    }
    
    fn refill(&self) {
        let now = Instant::now();
        let last = self.last_refill.load(Ordering::Relaxed);
        let elapsed = now.duration_since(last).as_secs_f64();
        
        let new_tokens = (elapsed * self.refill_rate as f64) as u64;
        if new_tokens > 0 {
            let current = self.tokens.load(Ordering::Relaxed);
            let new_total = (current + new_tokens).min(self.capacity);
            self.tokens.store(new_total, Ordering::Release);
            self.last_refill.store(now, Ordering::Release);
        }
    }
}
```

**Properties:**
- Thread-safe (lock-free)
- Adaptive to burst traffic
- Configurable rate and burst size

---

## Performance Summary

| Algorithm | Best | Average | Worst | Space |
|-----------|------|---------|-------|-------|
| Input detection | O(1) | O(n) | O(n) | O(1) |
| DNS resolution | O(1) cache | O(1) | O(timeout) | O(k) |
| Range to CIDR | O(log n) | O(log n) | O(log n) | O(log n) |
| CIDR aggregation | O(n log n) | O(n log n) | O(n log n) | O(n) |
| Overlap detection | O(log n) | O(log n + k) | O(n) | O(n) |
| LRU cache | O(1) | O(1) | O(1) | O(capacity) |
| Rate limiting | O(1) | O(1) | O(1) | O(1) |

Where:
- n = input size
- k = number of matches
- timeout = DNS timeout duration
