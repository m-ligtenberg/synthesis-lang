#[cfg(test)]
mod realtime_buffer_tests {
    use super::*;
    use std::thread;
    use std::time::{Duration, Instant};
    
    #[test]
    fn test_realtime_buffer_creation() {
        // Valid power-of-2 sizes should work
        assert!(RealtimeCircularBuffer::new(1024).is_ok());
        assert!(RealtimeCircularBuffer::new(2048).is_ok());
        assert!(RealtimeCircularBuffer::new(4096).is_ok());
        
        // Non-power-of-2 sizes should fail
        assert!(RealtimeCircularBuffer::new(1000).is_err());
        assert!(RealtimeCircularBuffer::new(3000).is_err());
        
        // Zero size should fail
        assert!(RealtimeCircularBuffer::new(0).is_err());
    }
    
    #[test]
    fn test_single_sample_write_read() {
        let buffer = RealtimeCircularBuffer::new(1024).unwrap();
        
        // Initially empty
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        assert_eq!(buffer.fill_level(), 0);
        
        // Write a sample
        assert!(buffer.write(0.5));
        assert!(!buffer.is_empty());
        assert_eq!(buffer.fill_level(), 1);
        
        // Read it back
        assert_eq!(buffer.read(), Some(0.5));
        assert!(buffer.is_empty());
        assert_eq!(buffer.fill_level(), 0);
        
        // Reading from empty buffer should return None
        assert_eq!(buffer.read(), None);
    }
    
    #[test]
    fn test_multiple_sample_operations() {
        let buffer = RealtimeCircularBuffer::new(8); // Small buffer for testing
        assert!(buffer.is_ok());
        let buffer = buffer.unwrap();
        
        // Write multiple samples
        let test_data = [1.0, 2.0, 3.0, 4.0, 5.0];
        for &sample in &test_data {
            assert!(buffer.write(sample));
        }
        
        assert_eq!(buffer.fill_level(), 5);
        
        // Read them back in order
        for &expected in &test_data {
            assert_eq!(buffer.read(), Some(expected));
        }
        
        assert!(buffer.is_empty());
    }
    
    #[test]
    fn test_buffer_overflow() {
        let buffer = RealtimeCircularBuffer::new(4).unwrap(); // 4-1=3 usable slots
        
        // Fill the buffer
        assert!(buffer.write(1.0));
        assert!(buffer.write(2.0));
        assert!(buffer.write(3.0));
        
        // Should be full now
        assert!(buffer.is_full());
        
        // Writing to full buffer should fail
        assert!(!buffer.write(4.0));
        
        // Read one sample to make space
        assert_eq!(buffer.read(), Some(1.0));
        
        // Now we can write again
        assert!(buffer.write(4.0));
    }
    
    #[test]
    fn test_write_read_slice() {
        let buffer = RealtimeCircularBuffer::new(1024).unwrap();
        
        // Test write_slice
        let input_data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let written = buffer.write_slice(&input_data);
        assert_eq!(written, input_data.len());
        
        // Test read_slice
        let mut output_data = [0.0; 5];
        let read_count = buffer.read_slice(&mut output_data);
        assert_eq!(read_count, 5);
        assert_eq!(output_data, input_data);
    }
    
    #[test]
    fn test_write_slice_overflow() {
        let buffer = RealtimeCircularBuffer::new(4).unwrap(); // 3 usable slots
        
        // Try to write more data than buffer can hold
        let large_data = [1.0, 2.0, 3.0, 4.0, 5.0]; // 5 samples
        let written = buffer.write_slice(&large_data);
        assert_eq!(written, 3); // Should only write 3 samples
        
        // Verify the first 3 samples were written
        assert_eq!(buffer.read(), Some(1.0));
        assert_eq!(buffer.read(), Some(2.0));
        assert_eq!(buffer.read(), Some(3.0));
        assert_eq!(buffer.read(), None); // Buffer should be empty now
    }
    
    #[test]
    fn test_read_slice_underflow() {
        let buffer = RealtimeCircularBuffer::new(1024).unwrap();
        
        // Write only 3 samples
        buffer.write_slice(&[1.0, 2.0, 3.0]);
        
        // Try to read 5 samples
        let mut output_data = [0.0; 5];
        let read_count = buffer.read_slice(&mut output_data);
        assert_eq!(read_count, 3); // Should only read 3 samples
        
        // First 3 elements should contain the data
        assert_eq!(output_data[0], 1.0);
        assert_eq!(output_data[1], 2.0);
        assert_eq!(output_data[2], 3.0);
        // Last 2 elements should still be 0.0 (unchanged)
        assert_eq!(output_data[3], 0.0);
        assert_eq!(output_data[4], 0.0);
    }
    
    #[test]
    fn test_circular_wrapping() {
        let buffer = RealtimeCircularBuffer::new(4).unwrap(); // 3 usable slots
        
        // Fill and empty the buffer multiple times to test wrapping
        for cycle in 0..5 {
            // Fill buffer
            for i in 0..3 {
                let value = (cycle * 3 + i) as f32;
                assert!(buffer.write(value));
            }
            
            // Empty buffer
            for i in 0..3 {
                let expected = (cycle * 3 + i) as f32;
                assert_eq!(buffer.read(), Some(expected));
            }
            
            assert!(buffer.is_empty());
        }
    }
    
    #[test]
    fn test_concurrent_single_producer_single_consumer() {
        let buffer = Arc::new(RealtimeCircularBuffer::new(1024).unwrap());
        let buffer_reader = buffer.clone();
        let buffer_writer = buffer.clone();
        
        let test_data: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        let expected_data = test_data.clone();
        
        // Producer thread
        let producer = thread::spawn(move || {
            for &sample in &test_data {
                while !buffer_writer.write(sample) {
                    thread::yield_now(); // Wait for space
                }
            }
        });
        
        // Consumer thread
        let consumer = thread::spawn(move || {
            let mut received_data = Vec::new();
            
            for _ in 0..1000 {
                loop {
                    if let Some(sample) = buffer_reader.read() {
                        received_data.push(sample);
                        break;
                    }
                    thread::yield_now(); // Wait for data
                }
            }
            
            received_data
        });
        
        // Wait for completion
        producer.join().unwrap();
        let received = consumer.join().unwrap();
        
        // Verify all data was transmitted correctly
        assert_eq!(received, expected_data);
    }
    
    #[test]
    fn test_mpmc_buffer() {
        let buffer = MpmcRealtimeBuffer::new(1024).unwrap();
        
        // Basic functionality test
        assert!(buffer.write(1.0));
        assert!(buffer.write(2.0));
        
        assert_eq!(buffer.read(), Some(1.0));
        assert_eq!(buffer.read(), Some(2.0));
        assert_eq!(buffer.read(), None);
    }
    
    #[test]
    fn test_shared_realtime_buffer() {
        let buffer = SharedRealtimeBuffer::new(1024).unwrap();
        
        // Write some data
        assert!(buffer.write(1.0));
        assert!(buffer.write(2.0));
        assert!(buffer.write(3.0));
        
        // Check stats
        let stats = buffer.get_stats();
        assert_eq!(stats.total_writes, 3);
        assert_eq!(stats.current_fill_level, 3);
        
        // Read data
        assert_eq!(buffer.read(), Some(1.0));
        assert_eq!(buffer.read(), Some(2.0));
        
        let stats = buffer.get_stats();
        assert_eq!(stats.total_reads, 2);
        assert_eq!(stats.current_fill_level, 1);
    }
    
    #[test]
    fn test_shared_buffer_overflow_underflow_stats() {
        let buffer = SharedRealtimeBuffer::new(4).unwrap(); // 3 usable slots
        
        // Fill buffer
        buffer.write(1.0);
        buffer.write(2.0);
        buffer.write(3.0);
        
        // Try to write to full buffer (should cause overrun)
        assert!(!buffer.write(4.0));
        
        // Read all data
        buffer.read();
        buffer.read();
        buffer.read();
        
        // Try to read from empty buffer (should cause underrun)
        assert_eq!(buffer.read(), None);
        
        let stats = buffer.get_stats();
        assert_eq!(stats.overruns, 1);
        assert_eq!(stats.underruns, 1);
    }
    
    #[test]
    fn test_buffer_pool() {
        let pool = RealtimeBufferPool::new(4, 1024).unwrap();
        
        assert_eq!(pool.buffer_count(), 4);
        assert_eq!(pool.buffer_size(), 1024);
        assert_eq!(pool.total_capacity(), 4 * 1023); // -1 for each buffer due to full/empty distinction
        
        // Test accessing buffers
        for i in 0..4 {
            let buffer = pool.get_buffer(i).unwrap();
            buffer.write((i + 1) as f32);
        }
        
        // Verify data
        for i in 0..4 {
            let buffer = pool.get_buffer(i).unwrap();
            assert_eq!(buffer.read(), Some((i + 1) as f32));
        }
        
        // Test aggregate stats
        let stats = pool.get_aggregate_stats();
        assert_eq!(stats.total_writes, 4);
        assert_eq!(stats.total_reads, 4);
    }
    
    #[test]
    fn test_unsafe_buffer() {
        let mut buffer = UnsafeRealtimeBuffer::new(1024).unwrap();
        
        // Test basic operations
        assert!(buffer.write(1.0));
        assert!(buffer.write(2.0));
        
        assert_eq!(buffer.read(), Some(1.0));
        assert_eq!(buffer.read(), Some(2.0));
        assert_eq!(buffer.read(), None);
        
        // Test overflow
        let mut buffer = UnsafeRealtimeBuffer::new(4).unwrap();
        assert!(buffer.write(1.0));
        assert!(buffer.write(2.0));
        assert!(buffer.write(3.0));
        assert!(!buffer.write(4.0)); // Should fail when full
    }
    
    #[test]
    fn test_performance_characteristics() {
        let buffer = RealtimeCircularBuffer::new(1024).unwrap();
        let iterations = 10000;
        
        // Measure write performance
        let start = Instant::now();
        for i in 0..iterations {
            buffer.write(i as f32);
        }
        let write_duration = start.elapsed();
        
        // Measure read performance
        let start = Instant::now();
        for _ in 0..iterations {
            buffer.read();
        }
        let read_duration = start.elapsed();
        
        // Performance should be very fast (sub-microsecond per operation)
        let avg_write_nanos = write_duration.as_nanos() / iterations as u128;
        let avg_read_nanos = read_duration.as_nanos() / iterations as u128;
        
        println!("Average write time: {}ns", avg_write_nanos);
        println!("Average read time: {}ns", avg_read_nanos);
        
        // These should be very fast operations (typically < 100ns each)
        assert!(avg_write_nanos < 1000, "Write operations too slow: {}ns", avg_write_nanos);
        assert!(avg_read_nanos < 1000, "Read operations too slow: {}ns", avg_read_nanos);
    }
    
    #[test]
    fn test_real_time_constraints() {
        // Test that operations are deterministic and don't allocate
        let buffer = RealtimeCircularBuffer::new(1024).unwrap();
        
        // Fill buffer halfway
        for i in 0..512 {
            buffer.write(i as f32);
        }
        
        // Perform mixed read/write operations
        // This simulates real-time audio processing pattern
        let start = Instant::now();
        for i in 0..1000 {
            // Read some samples
            let _sample1 = buffer.read();
            let _sample2 = buffer.read();
            
            // Write some samples  
            buffer.write((i * 2) as f32);
            buffer.write((i * 2 + 1) as f32);
        }
        let total_duration = start.elapsed();
        
        // Should complete very quickly (well under 1ms for real-time safety)
        let duration_micros = total_duration.as_micros();
        println!("Mixed operations took: {}μs", duration_micros);
        assert!(duration_micros < 500, "Operations took too long: {}μs", duration_micros);
    }
    
    #[test]
    fn test_thread_safety_stress() {
        let buffer = Arc::new(SharedRealtimeBuffer::new(1024).unwrap());
        let num_threads = 4;
        let operations_per_thread = 1000;
        
        let mut handles = Vec::new();
        
        // Spawn multiple producer threads
        for thread_id in 0..num_threads {
            let buffer = buffer.clone();
            let handle = thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let value = (thread_id * operations_per_thread + i) as f32;
                    while !buffer.write(value) {
                        thread::yield_now(); // Wait for space
                    }
                }
            });
            handles.push(handle);
        }
        
        // Spawn consumer thread
        let buffer_reader = buffer.clone();
        let consumer = thread::spawn(move || {
            let mut received = Vec::new();
            for _ in 0..(num_threads * operations_per_thread) {
                loop {
                    if let Some(sample) = buffer_reader.read() {
                        received.push(sample);
                        break;
                    }
                    thread::yield_now(); // Wait for data
                }
            }
            received
        });
        
        // Wait for all producers to finish
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Wait for consumer and verify we got all data
        let received = consumer.join().unwrap();
        assert_eq!(received.len(), num_threads * operations_per_thread);
        
        // Verify no data was lost (all values should be unique)
        let mut sorted_received = received;
        sorted_received.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut expected: Vec<f32> = (0..(num_threads * operations_per_thread))
            .map(|i| i as f32)
            .collect();
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        assert_eq!(sorted_received, expected);
    }
}