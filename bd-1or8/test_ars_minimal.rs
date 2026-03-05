// Test EARS parsing directly
include!("clarity-web/src/lattice/ears.rs");

fn main() {
    // Test the failing case
    let input = "When, the system shall do something";
    let result = parse_requirement(input);
    println!("Input: {}", input);
    println!("Result: {:?}", result);
    println!("Is error: {}", result.is_err());
    
    // Test a valid ubiquitous
    let input2 = "The system shall send email notifications";
    let result2 = parse_requirement(input2);
    println!("\nInput: {}", input2);
    println!("Result: {:?}", result2);
    println!("Is ok: {}", result2.is_ok());
}
