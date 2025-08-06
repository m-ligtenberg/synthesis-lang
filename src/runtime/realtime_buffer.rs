use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Lock-free circular buffer optimized for real-time audio processing
/// Provides single-producer, single-consumer (SPSC) access pattern
#[derive(Debug)]
pub struct RealtimeCircularBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_head: AtomicUsize,
    read_head: AtomicUsize,
    mask: usize, // For power-of-2 optimization
}

/// Multi-producer, multi-consumer (MPMC) lock-free circular buffer
/// Uses compare-and-swap for thread-safe operations at the cost of some performance
#[derive(Debug)]
pub struct MpmcRealtimeBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_head: AtomicUsize,
    read_head: AtomicUsize,
    mask: usize,
}

/// Buffer statistics for performance monitoring
#[derive(Debug, Clone, Default)]
pub struct BufferStats {
    pub total_writes: u64,
    pub total_reads: u64,
    pub overruns: u64,
    pub underruns: u64,
    pub current_fill_level: usize,
    pub peak_fill_level: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BufferError {
    BufferFull,
    BufferEmpty,
    InvalidSize,
    ReadOverrun,
    WriteOverrun,
}

impl RealtimeCircularBuffer {
    /// Create a new real-time circular buffer
    /// Size must be a power of 2 for optimal performance
    pub fn new(size: usize) -> Result<Self, BufferError> {
        if size == 0 || !size.is_power_of_two() {
            return Err(BufferError::InvalidSize);
        }
        
        Ok(RealtimeCircularBuffer {
            buffer: vec![0.0; size],
            capacity: size,
            write_head: AtomicUsize::new(0),
            read_head: AtomicUsize::new(0),
            mask: size - 1, // Power of 2 optimization
        })
    }
    
    /// Write a single sample to the buffer (lock-free, wait-free)
    /// Returns true if successful, false if buffer is full
    pub fn write(&self, sample: f32) -> bool {
        let write_pos = self.write_head.load(Ordering::Acquire);
        let read_pos = self.read_head.load(Ordering::Acquire);
        
        // Check if buffer is full (one slot is kept empty to distinguish full from empty)
        if (write_pos + 1) & self.mask == read_pos {
            return false; // Buffer full
        }
        
        // Write the sample
        self.buffer[write_pos & self.mask] = sample;
        
        // Advance write head (this makes the data visible to readers)
        self.write_head.store((write_pos + 1) & self.mask, Ordering::Release);
        
        true
    }
    
    /// Write multiple samples to the buffer
    /// Returns the number of samples actually written
    pub fn write_slice(&self, samples: &[f32]) -> usize {
        let mut written = 0;
        
        for &sample in samples {
            if !self.write(sample) {
                break; // Buffer full, stop writing
            }
            written += 1;
        }
        
        written
    }
    
    /// Read a single sample from the buffer (lock-free, wait-free)
    /// Returns Some(sample) if successful, None if buffer is empty
    pub fn read(&self) -> Option<f32> {
        let read_pos = self.read_head.load(Ordering::Acquire);
        let write_pos = self.write_head.load(Ordering::Acquire);
        
        // Check if buffer is empty
        if read_pos == write_pos {
            return None; // Buffer empty
        }
        
        // Read the sample
        let sample = self.buffer[read_pos & self.mask];
        
        // Advance read head
        self.read_head.store((read_pos + 1) & self.mask, Ordering::Release);
        
        Some(sample)
    }
    
    /// Read multiple samples from the buffer
    /// Returns the number of samples actually read
    pub fn read_slice(&self, samples: &mut [f32]) -> usize {
        let mut read_count = 0;
        
        for sample_slot in samples.iter_mut() {
            if let Some(sample) = self.read() {
                *sample_slot = sample;
                read_count += 1;
            } else {
                break; // Buffer empty, stop reading
            }
        }
        
        read_count
    }
    
    /// Get the current fill level of the buffer (0 to capacity-1)
    pub fn fill_level(&self) -> usize {
        let write_pos = self.write_head.load(Ordering::Acquire);
        let read_pos = self.read_head.load(Ordering::Acquire);
        
        (write_pos.wrapping_sub(read_pos)) & self.mask
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        let read_pos = self.read_head.load(Ordering::Acquire);
        let write_pos = self.write_head.load(Ordering::Acquire);
        read_pos == write_pos
    }
    
    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        let write_pos = self.write_head.load(Ordering::Acquire);
        let read_pos = self.read_head.load(Ordering::Acquire);
        (write_pos + 1) & self.mask == read_pos
    }
    
    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity - 1 // One slot is reserved to distinguish full from empty
    }
    
    /// Clear the buffer (not thread-safe, use only when no other threads are accessing)
    pub fn clear(&self) {
        self.read_head.store(0, Ordering::Release);
        self.write_head.store(0, Ordering::Release);
    }
}

impl MpmcRealtimeBuffer {
    /// Create a new multi-producer, multi-consumer real-time buffer
    pub fn new(size: usize) -> Result<Self, BufferError> {
        if size == 0 || !size.is_power_of_two() {
            return Err(BufferError::InvalidSize);
        }
        
        Ok(MpmcRealtimeBuffer {
            buffer: vec![0.0; size],
            capacity: size,
            write_head: AtomicUsize::new(0),
            read_head: AtomicUsize::new(0),
            mask: size - 1,
        })
    }
    
    /// Write a sample using compare-and-swap for thread safety
    pub fn write(&self, sample: f32) -> bool {
        loop {
            let write_pos = self.write_head.load(Ordering::Acquire);
            let read_pos = self.read_head.load(Ordering::Acquire);
            
            // Check if buffer is full
            if (write_pos + 1) & self.mask == read_pos {
                return false; // Buffer full
            }
            
            // Try to claim the write position
            match self.write_head.compare_exchange_weak(
                write_pos,
                (write_pos + 1) & self.mask,
                Ordering::AcqRel,
                Ordering::Acquire
            ) {
                Ok(_) => {
                    // Successfully claimed position, write the sample
                    self.buffer[write_pos & self.mask] = sample;
                    return true;
                }
                Err(_) => {
                    // Another thread got there first, retry
                    std::hint::spin_loop();
                    continue;
                }
            }
        }
    }
    
    /// Read a sample using compare-and-swap for thread safety
    pub fn read(&self) -> Option<f32> {
        loop {
            let read_pos = self.read_head.load(Ordering::Acquire);
            let write_pos = self.write_head.load(Ordering::Acquire);
            
            // Check if buffer is empty
            if read_pos == write_pos {
                return None; // Buffer empty
            }
            
            // Try to claim the read position
            match self.read_head.compare_exchange_weak(
                read_pos,
                (read_pos + 1) & self.mask,
                Ordering::AcqRel,
                Ordering::Acquire
            ) {
                Ok(_) => {
                    // Successfully claimed position, read the sample
                    let sample = self.buffer[read_pos & self.mask];
                    return Some(sample);
                }
                Err(_) => {
                    // Another thread got there first, retry
                    std::hint::spin_loop();
                    continue;
                }
            }
        }
    }
    
    /// Get current fill level (approximate due to concurrent access)
    pub fn fill_level(&self) -> usize {
        let write_pos = self.write_head.load(Ordering::Acquire);
        let read_pos = self.read_head.load(Ordering::Acquire);
        (write_pos.wrapping_sub(read_pos)) & self.mask
    }
    
    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity - 1
    }
}

/// Shared real-time buffer that can be safely shared between threads
#[derive(Debug, Clone)]
pub struct SharedRealtimeBuffer {
    inner: Arc<RealtimeCircularBuffer>,
    stats: Arc<std::sync::Mutex<BufferStats>>,
}

impl SharedRealtimeBuffer {
    pub fn new(size: usize) -> Result<Self, BufferError> {
        Ok(SharedRealtimeBuffer {
            inner: Arc::new(RealtimeCircularBuffer::new(size)?),
            stats: Arc::new(std::sync::Mutex::new(BufferStats::default())),
        })
    }
    
    pub fn write(&self, sample: f32) -> bool {
        let success = self.inner.write(sample);
        
        // Update stats (lock is held briefly)
        if let Ok(mut stats) = self.stats.try_lock() {
            if success {
                stats.total_writes += 1;
                let fill_level = self.inner.fill_level();
                stats.current_fill_level = fill_level;
                if fill_level > stats.peak_fill_level {
                    stats.peak_fill_level = fill_level;
                }
            } else {
                stats.overruns += 1;
            }
        }
        
        success
    }
    
    pub fn read(&self) -> Option<f32> {
        let result = self.inner.read();
        
        // Update stats (lock is held briefly)
        if let Ok(mut stats) = self.stats.try_lock() {
            if result.is_some() {
                stats.total_reads += 1;
                stats.current_fill_level = self.inner.fill_level();
            } else {
                stats.underruns += 1;
            }
        }
        
        result
    }
    
    pub fn write_slice(&self, samples: &[f32]) -> usize {
        let written = self.inner.write_slice(samples);
        
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.total_writes += written as u64;
            if written < samples.len() {
                stats.overruns += (samples.len() - written) as u64;
            }
            let fill_level = self.inner.fill_level();
            stats.current_fill_level = fill_level;
            if fill_level > stats.peak_fill_level {
                stats.peak_fill_level = fill_level;
            }
        }
        
        written
    }
    
    pub fn read_slice(&self, samples: &mut [f32]) -> usize {
        let read_count = self.inner.read_slice(samples);
        
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.total_reads += read_count as u64;
            if read_count < samples.len() {
                stats.underruns += (samples.len() - read_count) as u64;
            }
            stats.current_fill_level = self.inner.fill_level();
        }
        
        read_count
    }
    
    pub fn fill_level(&self) -> usize {
        self.inner.fill_level()
    }
    
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }
    
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    
    pub fn clear(&self) {
        self.inner.clear();
        
        if let Ok(mut stats) = self.stats.lock() {
            *stats = BufferStats::default();
        }
    }
    
    pub fn get_stats(&self) -> BufferStats {
        self.stats.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            let current_fill = self.inner.fill_level();
            *stats = BufferStats {
                current_fill_level: current_fill,
                peak_fill_level: current_fill,
                ..Default::default()
            };
        }
    }
}

/// Real-time buffer pool for managing multiple buffers efficiently
pub struct RealtimeBufferPool {
    buffers: Vec<SharedRealtimeBuffer>,
    buffer_size: usize,
}

impl RealtimeBufferPool {
    pub fn new(pool_size: usize, buffer_size: usize) -> Result<Self, BufferError> {
        if !buffer_size.is_power_of_two() {
            return Err(BufferError::InvalidSize);
        }
        
        let mut buffers = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            buffers.push(SharedRealtimeBuffer::new(buffer_size)?);
        }
        
        Ok(RealtimeBufferPool {
            buffers,
            buffer_size,
        })
    }
    
    pub fn get_buffer(&self, index: usize) -> Option<&SharedRealtimeBuffer> {
        self.buffers.get(index)
    }
    
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
    
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
    
    pub fn clear_all(&self) {
        for buffer in &self.buffers {
            buffer.clear();
        }
    }
    
    pub fn total_capacity(&self) -> usize {
        self.buffers.len() * self.buffer_size
    }
    
    pub fn get_aggregate_stats(&self) -> BufferStats {
        let mut total_stats = BufferStats::default();
        
        for buffer in &self.buffers {
            let stats = buffer.get_stats();
            total_stats.total_writes += stats.total_writes;
            total_stats.total_reads += stats.total_reads;
            total_stats.overruns += stats.overruns;
            total_stats.underruns += stats.underruns;
            total_stats.current_fill_level += stats.current_fill_level;
            total_stats.peak_fill_level = total_stats.peak_fill_level.max(stats.peak_fill_level);
        }
        
        total_stats
    }
}

// Unsafe optimized versions for ultra-low latency scenarios
// These bypass atomic operations entirely but require external synchronization

/// Ultra-low latency buffer for single-threaded use or with external synchronization
/// WARNING: Not thread-safe! Use only in single-threaded contexts or with external locks
pub struct UnsafeRealtimeBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_head: usize,
    read_head: usize,
    mask: usize,
}

impl UnsafeRealtimeBuffer {
    pub fn new(size: usize) -> Result<Self, BufferError> {
        if size == 0 || !size.is_power_of_two() {
            return Err(BufferError::InvalidSize);
        }
        
        Ok(UnsafeRealtimeBuffer {
            buffer: vec![0.0; size],
            capacity: size,
            write_head: 0,
            read_head: 0,
            mask: size - 1,
        })
    }
    
    /// Write sample - no atomic operations, maximum performance
    /// WARNING: Not thread-safe!
    pub fn write(&mut self, sample: f32) -> bool {
        if (self.write_head + 1) & self.mask == self.read_head {
            return false; // Buffer full
        }
        
        self.buffer[self.write_head & self.mask] = sample;
        self.write_head = (self.write_head + 1) & self.mask;
        true
    }
    
    /// Read sample - no atomic operations, maximum performance
    /// WARNING: Not thread-safe!
    pub fn read(&mut self) -> Option<f32> {
        if self.read_head == self.write_head {
            return None; // Buffer empty
        }
        
        let sample = self.buffer[self.read_head & self.mask];
        self.read_head = (self.read_head + 1) & self.mask;
        Some(sample)
    }
    
    pub fn fill_level(&self) -> usize {
        (self.write_head.wrapping_sub(self.read_head)) & self.mask
    }
    
    pub fn is_empty(&self) -> bool {
        self.read_head == self.write_head
    }
    
    pub fn is_full(&self) -> bool {
        (self.write_head + 1) & self.mask == self.read_head
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity - 1
    }
    
    pub fn clear(&mut self) {
        self.read_head = 0;
        self.write_head = 0;
    }
}