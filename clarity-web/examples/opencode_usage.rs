#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Example usage of the OpenCode extraction provider
//!
//! This example demonstrates how to use the OpenCode provider
//! to extract structured fields from unstructured text.

use clarity_web::providers::{
    ExtractionContext, ExtractionError, ExtractionProvider, FieldType,
    OpenCodeProvider, SchemaField,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), ExtractionError> {
    // Initialize the OpenCode provider
    let provider = OpenCodeProvider::new(
        "https://api.opencode.com".to_string(),
        "my-session-id-12345".to_string(),
    )?;

    println!("Provider: {}", provider.provider_name());
    println!("Session ID: {}", provider.session_id());
    println!();

    // Example 1: Simple extraction
    println!("=== Example 1: Simple Extraction ===");
    simple_extraction(&provider).await?;
    println!();

    // Example 2: Schema-based extraction
    println!("=== Example 2: Schema-Based Extraction ===");
    schema_extraction(&provider).await?;
    println!();

    // Example 3: Health check
    println!("=== Example 3: Health Check ===");
    health_check_example(&provider).await?;

    Ok(())
}

/// Example 1: Extract fields without a predefined schema
async fn simple_extraction(
    provider: &OpenCodeProvider,
) -> Result<(), ExtractionError> {
    let text = r#"
        John Doe
        Email: john.doe@example.com
        Phone: (555) 123-4567
        Address: 123 Main St, Anytown, CA 12345
    "#;

    let context = ExtractionContext {
        document_type: Some("contact_info".to_string()),
        locale: Some("en_US".to_string()),
        schema: None,
        extra: json!({
            "extraction_hints": ["email", "phone", "address"]
        }),
    };

    match provider.extract_fields(text, &context).await {
        Ok(result) => {
            println!("Extracted {} fields:", result.fields.len());
            println!("Overall confidence: {:.2}", result.confidence);
            println!("Processing time: {}ms", result.metadata.processing_duration_ms);
            println!();

            for field in result.fields {
                println!(
                    "  - {}: {} (confidence: {:.2})",
                    field.name, field.value, field.confidence
                );
                if let Some(justification) = field.justification {
                    println!("    Justification: {}", justification);
                }
            }
        }
        Err(e) => {
            println!("Extraction failed: {}", e);
        }
    }

    Ok(())
}

/// Example 2: Extract fields with a predefined schema
async fn schema_extraction(
    provider: &OpenCodeProvider,
) -> Result<(), ExtractionError> {
    let text = r#"
        INVOICE #12345
        Date: 2025-02-25
        Customer: Acme Corp

        Item 1: Widget A - $25.00 x 10 = $250.00
        Item 2: Widget B - $50.00 x 5 = $250.00

        Subtotal: $500.00
        Tax (10%): $50.00
        Total: $550.00
    "#;

    let schema = vec![
        SchemaField {
            name: "invoice_number".to_string(),
            field_type: FieldType::Text,
            required: true,
            description: Some("Invoice identifier".to_string()),
            options: None,
        },
        SchemaField {
            name: "invoice_date".to_string(),
            field_type: FieldType::Date,
            required: true,
            description: Some("Date of invoice".to_string()),
            options: None,
        },
        SchemaField {
            name: "customer_name".to_string(),
            field_type: FieldType::Text,
            required: true,
            description: Some("Customer company name".to_string()),
            options: None,
        },
        SchemaField {
            name: "total_amount".to_string(),
            field_type: FieldType::Currency,
            required: true,
            description: Some("Total amount due".to_string()),
            options: None,
        },
        SchemaField {
            name: "tax_amount".to_string(),
            field_type: FieldType::Currency,
            required: false,
            description: Some("Tax amount".to_string()),
            options: None,
        },
    ];

    let context = ExtractionContext {
        document_type: Some("invoice".to_string()),
        locale: Some("en_US".to_string()),
        schema: Some(schema.clone()),
        extra: json!({
            "currency": "USD"
        }),
    };

    match provider
        .extract_fields_with_schema(text, &schema, &context)
        .await
    {
        Ok(result) => {
            println!("Extracted {} fields from invoice:", result.fields.len());
            println!("Overall confidence: {:.2}", result.confidence);
            println!("Processing time: {}ms", result.metadata.processing_duration_ms);
            println!();

            for field in result.fields {
                println!(
                    "  - {}: {} (confidence: {:.2})",
                    field.name, field.value, field.confidence
                );
                if let Some(justification) = field.justification {
                    println!("    Justification: {}", justification);
                }
            }
        }
        Err(e) => {
            println!("Extraction failed: {}", e);
        }
    }

    Ok(())
}

/// Example 3: Check provider health
async fn health_check_example(
    provider: &OpenCodeProvider,
) -> Result<(), ExtractionError> {
    match provider.health_check().await {
        Ok(()) => {
            println!("Provider is healthy and ready!");
        }
        Err(e) => {
            println!("Provider health check failed: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = OpenCodeProvider::new(
            "https://api.opencode.com".to_string(),
            "test-session".to_string(),
        );

        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.provider_name(), "opencode");
        assert_eq!(provider.session_id(), "test-session");
    }

    #[test]
    fn test_schema_definition() {
        let schema = vec![
            SchemaField {
                name: "email".to_string(),
                field_type: FieldType::Email,
                required: true,
                description: Some("User email address".to_string()),
                options: None,
            },
            SchemaField {
                name: "status".to_string(),
                field_type: FieldType::Select,
                required: false,
                description: Some("Account status".to_string()),
                options: Some(vec!["active".to_string(), "inactive".to_string()]),
            },
        ];

        assert_eq!(schema.len(), 2);
        assert_eq!(schema[0].field_type, FieldType::Email);
        assert!(schema[0].required);
        assert_eq!(schema[1].field_type, FieldType::Select);
        assert!(!schema[1].required);
    }
}
