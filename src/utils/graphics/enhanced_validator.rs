use std::collections::HashMap;
use anyhow::{Result, Context, anyhow};
use thiserror::Error;

/// Enhanced GLSL validation errors with detailed information
#[derive(Debug, Clone, thiserror::Error)]
pub enum ShaderValidationError {
    #[error("Syntax error in {shader_type} shader at line {line}: {message}")]
    SyntaxError {
        shader_type: String,
        line: usize,
        message: String,
    },
    #[error("Semantic error in {shader_type} shader: {message}")]
    SemanticError {
        shader_type: String,
        message: String,
    },
    #[error("Version compatibility error: {message}")]
    VersionError {
        message: String,
    },
    #[error("Missing required uniform or attribute: {name}")]
    MissingBinding {
        name: String,
    },
}

/// Validation result with detailed feedback
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ShaderValidationError>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub performance_hints: Vec<String>,
}

/// Enhanced GLSL validator with comprehensive validation
pub struct EnhancedGLSLValidator {
    enable_advanced_checks: bool,
    target_versions: Vec<String>,
}

impl Default for EnhancedGLSLValidator {
    fn default() -> Self {
        Self {
            enable_advanced_checks: true,
            target_versions: vec![
                "330".to_string(),
                "400".to_string(),
                "450".to_string(),
            ],
        }
    }
}

impl EnhancedGLSLValidator {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_advanced_checks(mut self, enable: bool) -> Self {
        self.enable_advanced_checks = enable;
        self
    }

    /// Comprehensive shader validation
    pub fn validate_shader(
        &self,
        shader_code: &str,
        shader_type: &str,
        target_version: Option<&str>,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            performance_hints: Vec::new(),
        };

        // Basic validation first
        if let Err(e) = self.basic_validation(shader_code, shader_type) {
            result.errors.push(e);
            result.is_valid = false;
        }

        // Advanced validation if enabled
        if self.enable_advanced_checks {
            self.advanced_validation(shader_code, shader_type, &mut result);
        }

        // Performance analysis
        self.analyze_performance(shader_code, &mut result);

        // Generate suggestions
        self.generate_suggestions(shader_code, &mut result);

        Ok(result)
    }

    /// Basic validation (improved version of existing logic)
    fn basic_validation(&self, shader_code: &str, shader_type: &str) -> Result<(), ShaderValidationError> {
        let lines: Vec<&str> = shader_code.lines().collect();
        
        // Check for required GLSL version
        if !shader_code.contains("#version") {
            return Err(ShaderValidationError::SyntaxError {
                shader_type: shader_type.to_string(),
                line: 1,
                message: "Missing #version directive".to_string(),
            });
        }

        // Extract and validate version
        if let Some(version_line) = lines.iter().find(|line| line.trim_start().starts_with("#version")) {
            let version = version_line
                .trim_start()
                .strip_prefix("#version")
                .unwrap()
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("");
            
            if let Ok(version_num) = version.parse::<u32>() {
                if version_num < 330 {
                    return Err(ShaderValidationError::VersionError {
                        message: format!("GLSL version {} is too old, minimum supported is 330", version_num),
                    });
                }
            }
        }

        // Check for main function
        if !shader_code.contains("void main()") && !shader_code.contains("void main(void)") {
            return Err(ShaderValidationError::SyntaxError {
                shader_type: shader_type.to_string(),
                line: 0,
                message: "Missing main() function".to_string(),
            });
        }

        // Enhanced brace matching with line numbers
        let mut brace_stack = Vec::new();
        for (line_num, line) in lines.iter().enumerate() {
            for (char_pos, ch) in line.char_indices() {
                match ch {
                    '{' => brace_stack.push((line_num + 1, char_pos)),
                    '}' => {
                        if brace_stack.is_empty() {
                            return Err(ShaderValidationError::SyntaxError {
                                shader_type: shader_type.to_string(),
                                line: line_num + 1,
                                message: "Unmatched closing brace".to_string(),
                            });
                        }
                        brace_stack.pop();
                    }
                    _ => {}
                }
            }
        }

        if !brace_stack.is_empty() {
            let (line, _) = brace_stack.last().unwrap();
            return Err(ShaderValidationError::SyntaxError {
                shader_type: shader_type.to_string(),
                line: *line,
                message: "Unmatched opening brace".to_string(),
            });
        }

        Ok(())
    }

    /// Advanced validation with semantic checks
    fn advanced_validation(&self, shader_code: &str, shader_type: &str, result: &mut ValidationResult) {
        // Check for deprecated features
        let deprecated_warnings = self.check_deprecated_features(shader_code);
        result.warnings.extend(deprecated_warnings);

        // Check for potential issues
        let potential_issues = self.check_potential_issues(shader_code, shader_type);
        result.warnings.extend(potential_issues);

        // Check for performance issues
        let performance_issues = self.check_performance_issues(shader_code);
        result.warnings.extend(performance_issues);
    }

    /// Check for deprecated features
    fn check_deprecated_features(&self, shader_code: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if shader_code.contains("texture2D") {
            warnings.push("Using deprecated texture2D() - use texture() instead".to_string());
        }
        
        if shader_code.contains("varying") {
            warnings.push("Using deprecated 'varying' - use 'in'/'out' instead".to_string());
        }
        
        if shader_code.contains("attribute") {
            warnings.push("Using deprecated 'attribute' - use 'in' instead".to_string());
        }
        
        if shader_code.contains("gl_FragColor") {
            warnings.push("Using deprecated gl_FragColor - declare custom output variable".to_string());
        }

        warnings
    }

    /// Check for potential issues
    fn check_potential_issues(&self, shader_code: &str, shader_type: &str) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check for precision qualifiers
        if !shader_code.contains("precision") {
            issues.push("No precision qualifier specified - consider adding 'precision mediump float;'".to_string());
        }
        
        // Check for proper output variable in fragment shader
        if shader_type == "fragment" && !shader_code.contains("out vec4") && !shader_code.contains("gl_FragColor") {
            issues.push("Fragment shader should declare output variable".to_string());
        }

        issues
    }

    /// Check for performance issues
    fn check_performance_issues(&self, shader_code: &str) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check for expensive operations
        if shader_code.contains("pow") || shader_code.contains("exp") || shader_code.contains("log") {
            issues.push("Expensive math functions detected - consider approximations for better performance".to_string());
        }
        
        // Check for loops
        if shader_code.contains("for") || shader_code.contains("while") {
            issues.push("Loops detected - ensure they have reasonable bounds for mobile compatibility".to_string());
        }
        
        // Check for excessive branching
        let if_count = shader_code.matches("if").count();
        if if_count > 3 {
            issues.push(format!("High branching count ({}), may cause GPU divergence", if_count));
        }

        issues
    }

    /// Analyze shader performance characteristics
    fn analyze_performance(&self, shader_code: &str, result: &mut ValidationResult) {
        let mut hints = Vec::new();
        
        // Check for expensive operations
        if shader_code.contains("texture2D") || shader_code.contains("texture") {
            let texture_count = shader_code.matches("texture").count();
            if texture_count > 4 {
                hints.push(format!("High texture read count ({}), consider reducing for better performance", texture_count));
            }
        }

        // Check for loops
        if shader_code.contains("for") || shader_code.contains("while") {
            hints.push("Loops detected - ensure they have reasonable bounds for mobile compatibility".to_string());
        }

        // Check for complex math operations
        if shader_code.contains("pow") || shader_code.contains("exp") || shader_code.contains("log") {
            hints.push("Expensive math functions detected - consider approximations for better performance".to_string());
        }

        // Check for excessive branching
        let if_count = shader_code.matches("if").count();
        if if_count > 3 {
            hints.push(format!("High branching count ({}), may cause GPU divergence", if_count));
        }

        result.performance_hints = hints;
    }

    /// Generate optimization suggestions
    fn generate_suggestions(&self, shader_code: &str, result: &mut ValidationResult) {
        let mut suggestions = Vec::new();
        
        // Suggest modern alternatives
        if shader_code.contains("texture2D") {
            suggestions.push("Replace texture2D() with texture() for modern GLSL".to_string());
        }
        
        if shader_code.contains("gl_FragColor") {
            suggestions.push("Replace gl_FragColor with custom 'out vec4 fragColor;' declaration".to_string());
        }
        
        if shader_code.contains("varying") {
            suggestions.push("Replace 'varying' with 'in'/'out' for modern GLSL".to_string());
        }
        
        if shader_code.contains("attribute") {
            suggestions.push("Replace 'attribute' with 'in' for modern GLSL".to_string());
        }
        
        // Performance suggestions
        if shader_code.contains("pow") {
            suggestions.push("Consider using faster alternatives to pow() for integer exponents".to_string());
        }
        
        if shader_code.matches("if").count() > 2 {
            suggestions.push("Consider using step() or smoothstep() instead of multiple if statements".to_string());
        }

        result.suggestions = suggestions;
    }
}
