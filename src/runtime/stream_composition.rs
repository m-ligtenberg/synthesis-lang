use crate::runtime::types::{DataType, Value};
use crate::runtime::streams::{StreamManager, StreamTask, TaskType, TaskPriority};
use crate::errors::ErrorKind;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

/// Stream composition engine for complex routing and processing
#[derive(Debug)]
pub struct StreamCompositionEngine {
    pub connections: HashMap<String, Vec<StreamConnection>>,
    pub processing_graph: StreamGraph,
    pub composition_rules: Vec<CompositionRule>,
}

/// Represents a connection between two streams
#[derive(Debug, Clone, PartialEq)]
pub struct StreamConnection {
    pub source: String,
    pub destination: String,
    pub connection_type: ConnectionType,
    pub transform: Option<StreamTransform>,
    pub routing: RoutingConfig,
}

/// Types of stream connections
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    Direct,        // Simple 1:1 connection
    Split,         // 1:many connection (broadcast)
    Merge,         // Many:1 connection (mix)
    Chain,         // Sequential processing
    Parallel,      // Parallel processing
    Conditional,   // Connection based on conditions
}

/// Stream transformation applied during connection
#[derive(Debug, Clone, PartialEq)]
pub struct StreamTransform {
    pub transform_id: String,
    pub parameters: HashMap<String, Value>,
    pub bypass: bool,
}

/// Routing configuration for connection
#[derive(Debug, Clone, PartialEq)]
pub struct RoutingConfig {
    pub channel_mapping: Option<ChannelMapping>,
    pub gain: f32,
    pub delay_samples: usize,
    pub enabled: bool,
}

/// Channel mapping for multi-channel streams
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMapping {
    pub input_channels: Vec<usize>,
    pub output_channels: Vec<usize>,
    pub mix_matrix: Option<Vec<Vec<f32>>>, // For complex channel mixing
}

/// Directed graph representation of stream connections
#[derive(Debug)]
pub struct StreamGraph {
    pub nodes: HashMap<String, StreamNode>,
    pub edges: Vec<StreamEdge>,
    pub execution_order: Vec<String>, // Topologically sorted execution order
}

/// Node in the stream graph
#[derive(Debug, Clone)]
pub struct StreamNode {
    pub stream_name: String,
    pub node_type: StreamNodeType,
    pub processing_priority: TaskPriority,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

/// Types of stream nodes
#[derive(Debug, Clone, PartialEq)]
pub enum StreamNodeType {
    Source,      // Input streams
    Sink,        // Output streams
    Transform,   // Processing streams
    Router,      // Routing/switching streams
    Buffer,      // Buffering streams
}

/// Edge in the stream graph
#[derive(Debug, Clone)]
pub struct StreamEdge {
    pub from: String,
    pub to: String,
    pub connection: StreamConnection,
    pub weight: f32, // For weighted routing
}

/// Rules for automatic stream composition
#[derive(Debug, Clone)]
pub struct CompositionRule {
    pub name: String,
    pub condition: CompositionCondition,
    pub action: CompositionAction,
    pub priority: u32,
    pub enabled: bool,
}

/// Conditions for composition rules
#[derive(Debug, Clone, PartialEq)]
pub enum CompositionCondition {
    StreamTypeMatch { source_type: DataType, destination_type: DataType },
    NamePattern { pattern: String },
    MetadataMatch { key: String, value: Value },
    BufferLevel { stream: String, threshold: f32, comparison: Comparison },
    StreamActivity { stream: String, active: bool },
    Always,
}

/// Actions for composition rules
#[derive(Debug, Clone, PartialEq)]
pub enum CompositionAction {
    Connect { connection_type: ConnectionType, transform: Option<StreamTransform> },
    Disconnect,
    SetGain { gain: f32 },
    SetDelay { delay_samples: usize },
    EnableBypass { bypass: bool },
    CreateBuffer { size: usize },
}

/// Comparison operators for conditions
#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    Greater,
    Less,
    Equal,
    GreaterEqual,
    LessEqual,
}

impl StreamCompositionEngine {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            processing_graph: StreamGraph::new(),
            composition_rules: Vec::new(),
        }
    }
    
    /// Create a direct connection between two streams
    pub fn connect_direct(&mut self, source: String, destination: String, gain: f32) -> crate::Result<()> {
        let connection = StreamConnection {
            source: source.clone(),
            destination: destination.clone(),
            connection_type: ConnectionType::Direct,
            transform: None,
            routing: RoutingConfig {
                channel_mapping: None,
                gain,
                delay_samples: 0,
                enabled: true,
            },
        };
        
        self.add_connection(connection)?;
        self.rebuild_graph()?;
        Ok(())
    }
    
    /// Create a split connection (1:many)
    pub fn connect_split(&mut self, source: String, destinations: Vec<String>, gains: Vec<f32>) -> crate::Result<()> {
        if destinations.len() != gains.len() {
            return Err(crate::SynthesisError::new(ErrorKind::InvalidArgument,
                "Number of destinations must match number of gains".to_string()));
        }
        
        for (dest, gain) in destinations.iter().zip(gains.iter()) {
            let connection = StreamConnection {
                source: source.clone(),
                destination: dest.clone(),
                connection_type: ConnectionType::Split,
                transform: None,
                routing: RoutingConfig {
                    channel_mapping: None,
                    gain: *gain,
                    delay_samples: 0,
                    enabled: true,
                },
            };
            
            self.add_connection(connection)?;
        }
        
        self.rebuild_graph()?;
        Ok(())
    }
    
    /// Create a merge connection (many:1)
    pub fn connect_merge(&mut self, sources: Vec<String>, destination: String, mix_gains: Vec<f32>) -> crate::Result<()> {
        if sources.len() != mix_gains.len() {
            return Err(crate::SynthesisError::new(ErrorKind::InvalidArgument,
                "Number of sources must match number of mix gains".to_string()));
        }
        
        for (source, gain) in sources.iter().zip(mix_gains.iter()) {
            let connection = StreamConnection {
                source: source.clone(),
                destination: destination.clone(),
                connection_type: ConnectionType::Merge,
                transform: None,
                routing: RoutingConfig {
                    channel_mapping: None,
                    gain: *gain,
                    delay_samples: 0,
                    enabled: true,
                },
            };
            
            self.add_connection(connection)?;
        }
        
        self.rebuild_graph()?;
        Ok(())
    }
    
    /// Create a processing chain
    pub fn connect_chain(&mut self, streams: Vec<String>, transforms: Vec<StreamTransform>) -> crate::Result<()> {
        if streams.len() < 2 {
            return Err(crate::SynthesisError::new(ErrorKind::InvalidArgument,
                "Chain requires at least 2 streams".to_string()));
        }
        
        for i in 0..streams.len() - 1 {
            let transform = transforms.get(i).cloned();
            let connection = StreamConnection {
                source: streams[i].clone(),
                destination: streams[i + 1].clone(),
                connection_type: ConnectionType::Chain,
                transform,
                routing: RoutingConfig {
                    channel_mapping: None,
                    gain: 1.0,
                    delay_samples: 0,
                    enabled: true,
                },
            };
            
            self.add_connection(connection)?;
        }
        
        self.rebuild_graph()?;
        Ok(())
    }
    
    /// Create parallel processing paths
    pub fn connect_parallel(&mut self, input: String, outputs: Vec<String>, merge_back: Option<String>) -> crate::Result<()> {
        // Split to parallel paths
        let gains = vec![1.0; outputs.len()];
        self.connect_split(input, outputs.clone(), gains)?;
        
        // Optionally merge back
        if let Some(merge_destination) = merge_back {
            let mix_gains = vec![1.0 / outputs.len() as f32; outputs.len()]; // Equal mixing
            self.connect_merge(outputs, merge_destination, mix_gains)?;
        }
        
        Ok(())
    }
    
    /// Add a connection to the composition engine
    fn add_connection(&mut self, connection: StreamConnection) -> crate::Result<()> {
        let source = connection.source.clone();
        
        self.connections
            .entry(source)
            .or_insert_with(Vec::new)
            .push(connection);
        
        Ok(())
    }
    
    /// Remove a connection
    pub fn disconnect(&mut self, source: &str, destination: &str) -> crate::Result<bool> {
        if let Some(connections) = self.connections.get_mut(source) {
            let initial_len = connections.len();
            connections.retain(|conn| conn.destination != destination);
            
            if connections.len() != initial_len {
                self.rebuild_graph()?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Rebuild the processing graph from connections
    fn rebuild_graph(&mut self) -> crate::Result<()> {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        
        // Collect all unique stream names
        let mut all_streams = HashSet::new();
        for connections in self.connections.values() {
            for connection in connections {
                all_streams.insert(connection.source.clone());
                all_streams.insert(connection.destination.clone());
            }
        }
        
        // Create nodes
        for stream_name in &all_streams {
            let node_type = self.determine_node_type(stream_name);
            let priority = match node_type {
                StreamNodeType::Source => TaskPriority::Critical,
                StreamNodeType::Sink => TaskPriority::High,
                StreamNodeType::Transform => TaskPriority::Medium,
                _ => TaskPriority::Low,
            };
            
            let node = StreamNode {
                stream_name: stream_name.clone(),
                node_type,
                processing_priority: priority,
                dependencies: Vec::new(),
                dependents: Vec::new(),
            };
            
            nodes.insert(stream_name.clone(), node);
        }
        
        // Create edges and update dependencies
        for connections in self.connections.values() {
            for connection in connections {
                let edge = StreamEdge {
                    from: connection.source.clone(),
                    to: connection.destination.clone(),
                    connection: connection.clone(),
                    weight: connection.routing.gain,
                };
                
                // Update dependencies
                if let Some(dest_node) = nodes.get_mut(&connection.destination) {
                    dest_node.dependencies.push(connection.source.clone());
                }
                
                if let Some(src_node) = nodes.get_mut(&connection.source) {
                    src_node.dependents.push(connection.destination.clone());
                }
                
                edges.push(edge);
            }
        }
        
        // Calculate execution order using topological sort
        let execution_order = self.topological_sort(&nodes)?;
        
        self.processing_graph = StreamGraph {
            nodes,
            edges,
            execution_order,
        };
        
        Ok(())
    }
    
    /// Determine the type of a stream node based on its name and connections
    fn determine_node_type(&self, stream_name: &str) -> StreamNodeType {
        let has_inputs = self.connections.values()
            .any(|conns| conns.iter().any(|c| c.destination == stream_name));
        
        let has_outputs = self.connections.contains_key(stream_name);
        
        match (has_inputs, has_outputs) {
            (false, true) => StreamNodeType::Source,
            (true, false) => StreamNodeType::Sink,
            (true, true) => {
                // Check if it's a transform by looking at naming patterns
                if stream_name.contains("transform") || stream_name.contains("filter") || 
                   stream_name.contains("effect") || stream_name.contains("process") {
                    StreamNodeType::Transform
                } else if stream_name.contains("buffer") {
                    StreamNodeType::Buffer
                } else {
                    StreamNodeType::Router
                }
            }
            (false, false) => StreamNodeType::Buffer, // Isolated stream
        }
    }
    
    /// Perform topological sort to determine execution order
    fn topological_sort(&self, nodes: &HashMap<String, StreamNode>) -> crate::Result<Vec<String>> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut result = Vec::new();
        
        for node_name in nodes.keys() {
            if !visited.contains(node_name) {
                self.topological_visit(node_name, nodes, &mut visited, &mut temp_visited, &mut result)?;
            }
        }
        
        result.reverse(); // Reverse to get correct order
        Ok(result)
    }
    
    fn topological_visit(
        &self,
        node: &str,
        nodes: &HashMap<String, StreamNode>,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> crate::Result<()> {
        if temp_visited.contains(node) {
            return Err(crate::SynthesisError::new(ErrorKind::InvalidArgument,
                format!("Circular dependency detected involving stream '{}'", node)));
        }
        
        if visited.contains(node) {
            return Ok(());
        }
        
        temp_visited.insert(node.to_string());
        
        if let Some(stream_node) = nodes.get(node) {
            for dependent in &stream_node.dependents {
                self.topological_visit(dependent, nodes, visited, temp_visited, result)?;
            }
        }
        
        temp_visited.remove(node);
        visited.insert(node.to_string());
        result.push(node.to_string());
        
        Ok(())
    }
    
    /// Process the stream composition graph
    pub fn process_composition(&mut self, stream_manager: &mut StreamManager) -> crate::Result<()> {
        for stream_name in &self.processing_graph.execution_order.clone() {
            if let Some(connections) = self.connections.get(stream_name) {
                for connection in connections {
                    self.process_connection(connection, stream_manager)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single connection
    fn process_connection(&self, connection: &StreamConnection, stream_manager: &mut StreamManager) -> crate::Result<()> {
        if !connection.routing.enabled {
            return Ok(());
        }
        
        match connection.connection_type {
            ConnectionType::Direct => {
                self.process_direct_connection(connection, stream_manager)
            }
            ConnectionType::Split => {
                self.process_split_connection(connection, stream_manager)
            }
            ConnectionType::Merge => {
                self.process_merge_connection(connection, stream_manager)
            }
            ConnectionType::Chain => {
                self.process_chain_connection(connection, stream_manager)
            }
            ConnectionType::Parallel => {
                self.process_parallel_connection(connection, stream_manager)
            }
            ConnectionType::Conditional => {
                self.process_conditional_connection(connection, stream_manager)
            }
        }
    }
    
    fn process_direct_connection(&self, connection: &StreamConnection, stream_manager: &mut StreamManager) -> crate::Result<()> {
        // Read from source
        let source_data = stream_manager.read_from_stream(&connection.source, 128)?;
        
        // Apply gain
        let processed_data: Vec<f32> = source_data.iter()
            .map(|&sample| sample * connection.routing.gain)
            .collect();
        
        // Apply delay if specified
        let final_data = if connection.routing.delay_samples > 0 {
            let mut delayed = vec![0.0; connection.routing.delay_samples];
            delayed.extend(processed_data);
            delayed
        } else {
            processed_data
        };
        
        // Apply transform if present
        let transformed_data = if let Some(ref transform) = connection.transform {
            if !transform.bypass {
                self.apply_transform(&final_data, transform)?
            } else {
                final_data
            }
        } else {
            final_data
        };
        
        // Write to destination
        stream_manager.write_to_stream(&connection.destination, transformed_data)?;
        
        Ok(())
    }
    
    fn process_split_connection(&self, connection: &StreamConnection, stream_manager: &mut StreamManager) -> crate::Result<()> {
        // Split is handled by having multiple connections from the same source
        self.process_direct_connection(connection, stream_manager)
    }
    
    fn process_merge_connection(&self, connection: &StreamConnection, stream_manager: &mut StreamManager) -> crate::Result<()> {
        // For merge, we need to accumulate data from all sources
        // This is a simplified version - a full implementation would need coordination
        self.process_direct_connection(connection, stream_manager)
    }
    
    fn process_chain_connection(&self, connection: &StreamConnection, stream_manager: &mut StreamManager) -> crate::Result<()> {
        // Chain processing is the same as direct connection with potential transforms
        self.process_direct_connection(connection, stream_manager)
    }
    
    fn process_parallel_connection(&self, connection: &StreamConnection, stream_manager: &mut StreamManager) -> crate::Result<()> {
        // Parallel processing copies data to multiple paths
        self.process_direct_connection(connection, stream_manager)
    }
    
    fn process_conditional_connection(&self, connection: &StreamConnection, stream_manager: &mut StreamManager) -> crate::Result<()> {
        // Check conditions before processing
        // For now, just process as direct connection
        self.process_direct_connection(connection, stream_manager)
    }
    
    fn apply_transform(&self, data: &[f32], transform: &StreamTransform) -> crate::Result<Vec<f32>> {
        // Apply transform based on type and parameters
        // This is a placeholder - real implementation would dispatch to transform functions
        match transform.transform_id.as_str() {
            "gain" => {
                if let Some(Value::Float(amount)) = transform.parameters.get("amount") {
                    Ok(data.iter().map(|&sample| sample * (*amount as f32)).collect())
                } else {
                    Ok(data.to_vec())
                }
            }
            "lowpass" => {
                // Simple low-pass filter
                let mut filtered = Vec::with_capacity(data.len());
                let mut prev = 0.0;
                let cutoff = if let Some(Value::Float(c)) = transform.parameters.get("cutoff") {
                    *c as f32
                } else {
                    0.5
                };
                
                for &sample in data {
                    let filtered_sample = prev + cutoff * (sample - prev);
                    filtered.push(filtered_sample);
                    prev = filtered_sample;
                }
                Ok(filtered)
            }
            _ => Ok(data.to_vec()) // Unknown transform, pass through
        }
    }
    
    /// Add a composition rule
    pub fn add_composition_rule(&mut self, rule: CompositionRule) {
        self.composition_rules.push(rule);
        // Sort by priority (higher priority first)
        self.composition_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Apply composition rules
    pub fn apply_composition_rules(&mut self, stream_manager: &StreamManager) -> crate::Result<()> {
        for rule in &self.composition_rules.clone() {
            if rule.enabled && self.evaluate_condition(&rule.condition, stream_manager)? {
                self.execute_action(&rule.action, stream_manager)?;
            }
        }
        Ok(())
    }
    
    fn evaluate_condition(&self, condition: &CompositionCondition, stream_manager: &StreamManager) -> crate::Result<bool> {
        match condition {
            CompositionCondition::Always => Ok(true),
            CompositionCondition::StreamTypeMatch { source_type, destination_type } => {
                // Check if any connections match the type pattern
                Ok(self.connections.values().any(|conns| {
                    conns.iter().any(|conn| {
                        // In a full implementation, we'd check actual stream types
                        // For now, just return true as a placeholder
                        true
                    })
                }))
            }
            CompositionCondition::StreamActivity { stream, active } => {
                if let Some(stream_info) = stream_manager.get_stream_info(stream) {
                    Ok(stream_info.is_active == *active)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false) // Other conditions not implemented yet
        }
    }
    
    fn execute_action(&mut self, action: &CompositionAction, _stream_manager: &StreamManager) -> crate::Result<()> {
        match action {
            CompositionAction::SetGain { gain } => {
                // Apply gain to all connections
                for connections in self.connections.values_mut() {
                    for connection in connections {
                        connection.routing.gain = *gain;
                    }
                }
                Ok(())
            }
            CompositionAction::EnableBypass { bypass } => {
                // Set bypass on all transforms
                for connections in self.connections.values_mut() {
                    for connection in connections {
                        if let Some(ref mut transform) = connection.transform {
                            transform.bypass = *bypass;
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()) // Other actions not implemented yet
        }
    }
    
    /// Get composition statistics
    pub fn get_composition_stats(&self) -> CompositionStats {
        CompositionStats {
            total_connections: self.connections.values().map(|v| v.len()).sum(),
            total_streams: self.processing_graph.nodes.len(),
            execution_order_length: self.processing_graph.execution_order.len(),
            rules_count: self.composition_rules.len(),
            active_connections: self.connections.values()
                .flat_map(|v| v.iter())
                .filter(|c| c.routing.enabled)
                .count(),
        }
    }
}

/// Statistics about the composition engine
#[derive(Debug, Clone)]
pub struct CompositionStats {
    pub total_connections: usize,
    pub total_streams: usize,
    pub execution_order_length: usize,
    pub rules_count: usize,
    pub active_connections: usize,
}

impl StreamGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            execution_order: Vec::new(),
        }
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            channel_mapping: None,
            gain: 1.0,
            delay_samples: 0,
            enabled: true,
        }
    }
}

impl Default for StreamCompositionEngine {
    fn default() -> Self {
        Self::new()
    }
}