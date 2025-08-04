use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitValue {
    pub value: f64,
    pub unit: Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    // Time units
    Second,
    Millisecond,
    
    // Spatial units
    Pixel,
    Percent,
    
    // Angular units
    Degree,
    Radian,
    
    // Frequency units
    Hertz,
    Kilohertz,
    
    // Dimensionless
    Scalar,
}

impl Unit {
    pub fn from_string(unit_str: &str) -> Option<Unit> {
        match unit_str {
            "s" => Some(Unit::Second),
            "ms" => Some(Unit::Millisecond),
            "px" => Some(Unit::Pixel),
            "%" | "percent" => Some(Unit::Percent),
            "degrees" => Some(Unit::Degree),
            "radians" => Some(Unit::Radian),
            "Hz" => Some(Unit::Hertz),
            "kHz" => Some(Unit::Kilohertz),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            Unit::Second => "s",
            Unit::Millisecond => "ms",
            Unit::Pixel => "px",
            Unit::Percent => "%",
            Unit::Degree => "degrees",
            Unit::Radian => "radians",
            Unit::Hertz => "Hz",
            Unit::Kilohertz => "kHz",
            Unit::Scalar => "",
        }
    }
    
    pub fn is_compatible(&self, other: &Unit) -> bool {
        use Unit::*;
        match (self, other) {
            // Time units are compatible
            (Second, Millisecond) | (Millisecond, Second) => true,
            
            // Angular units are compatible
            (Degree, Radian) | (Radian, Degree) => true,
            
            // Frequency units are compatible
            (Hertz, Kilohertz) | (Kilohertz, Hertz) => true,
            
            // Same units are always compatible
            (a, b) if a == b => true,
            
            // Scalars are compatible with anything
            (Scalar, _) | (_, Scalar) => true,
            
            _ => false,
        }
    }
    
    pub fn conversion_factor(&self, to: &Unit) -> Option<f64> {
        use Unit::*;
        match (self, to) {
            // Same unit
            (a, b) if a == b => Some(1.0),
            
            // Time conversions
            (Second, Millisecond) => Some(1000.0),
            (Millisecond, Second) => Some(0.001),
            
            // Angular conversions
            (Degree, Radian) => Some(std::f64::consts::PI / 180.0),
            (Radian, Degree) => Some(180.0 / std::f64::consts::PI),
            
            // Frequency conversions
            (Hertz, Kilohertz) => Some(0.001),
            (Kilohertz, Hertz) => Some(1000.0),
            
            // To/from scalar
            (Scalar, _) | (_, Scalar) => Some(1.0),
            
            _ => None,
        }
    }
}

impl UnitValue {
    pub fn new(value: f64, unit: Unit) -> Self {
        Self { value, unit }
    }
    
    pub fn from_string(value: f64, unit_str: &str) -> Option<Self> {
        Unit::from_string(unit_str).map(|unit| Self::new(value, unit))
    }
    
    pub fn convert_to(&self, target_unit: &Unit) -> Option<UnitValue> {
        if let Some(factor) = self.unit.conversion_factor(target_unit) {
            Some(UnitValue::new(self.value * factor, target_unit.clone()))
        } else {
            None
        }
    }
    
    pub fn add(&self, other: &UnitValue) -> Option<UnitValue> {
        if self.unit.is_compatible(&other.unit) {
            if let Some(converted) = other.convert_to(&self.unit) {
                Some(UnitValue::new(self.value + converted.value, self.unit.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn subtract(&self, other: &UnitValue) -> Option<UnitValue> {
        if self.unit.is_compatible(&other.unit) {
            if let Some(converted) = other.convert_to(&self.unit) {
                Some(UnitValue::new(self.value - converted.value, self.unit.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn multiply(&self, scalar: f64) -> UnitValue {
        UnitValue::new(self.value * scalar, self.unit.clone())
    }
    
    pub fn divide(&self, scalar: f64) -> Option<UnitValue> {
        if scalar != 0.0 {
            Some(UnitValue::new(self.value / scalar, self.unit.clone()))
        } else {
            None
        }
    }
    
    // Convert to base unit value (for calculations)
    pub fn to_base_value(&self) -> f64 {
        match &self.unit {
            // Time base: seconds
            Unit::Millisecond => self.value * 0.001,
            
            // Angular base: radians
            Unit::Degree => self.value * std::f64::consts::PI / 180.0,
            
            // Frequency base: Hz
            Unit::Kilohertz => self.value * 1000.0,
            
            // Everything else is already in base units
            _ => self.value,
        }
    }
}