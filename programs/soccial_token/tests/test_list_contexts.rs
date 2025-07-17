/// ======================================================================
/// Test that scans all `context.rs` files in `src/**/context.rs`
/// and validates whether each Anchor context with `#[derive(Accounts)]`
/// includes at least one signer.
///
/// If a context includes a signer, it's assumed it should be tested
/// for signer enforcement. Contexts without signers must be explicitly
/// whitelisted if intentional.
///
/// Author: Paulo Rodrigues  
/// Project: Soccial Token  
/// Website: https://www.soccial.com/thetoken  
/// ======================================================================

use std::collections::HashSet;
use std::fs;
use regex::Regex;
use walkdir::WalkDir;

#[test]
fn test_all_contexts_have_secure_signer() {
    // Define context file pattern
    let file_pattern = "context.rs";

    // Define known files to ignore (these will be listed but not fail the test)
    let ignored_files: HashSet<&str> = [
        "src/market/context.rs",
        // Add more here if needed
    ].iter().cloned().collect();

    // Regex to capture #[derive(Accounts)] pub struct X<T> { ... }
    let struct_re = Regex::new(
        r#"(?s)#\[.*?derive\s*\(\s*Accounts\s*\).*?]\s*pub\s+struct\s+(\w+)\s*<[^>]*>\s*\{(.*?)\}"#
    ).expect("Failed to compile struct regex");

    // Match #[account(signer)] followed by pub line
    let signer_attr_re = Regex::new(
        r"(?m)#\[account\([^\]]*signer[^\]]*\)\]\s*\n\s*pub\s+\w+\s*:"
    ).expect("Failed to compile signer attribute regex");

    // Match pub x: Signer<'info>
    let signer_type_re = Regex::new(
        r"(?m)^\s*pub\s+\w+\s*:\s*Signer\s*<\s*'info\s*>\s*,?\s*$"
    ).expect("Failed to compile signer type regex");

    let mut insecure_contexts = vec![];

    // Walk recursively under src/
    for entry in WalkDir::new("src")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == file_pattern)
    {
        let path = entry.path();
        let file_name = path.display().to_string();

        let content = fs::read_to_string(path).expect("Failed to read file");

        for struct_cap in struct_re.captures_iter(&content) {
            let struct_name = struct_cap.get(1).unwrap().as_str();
            let struct_body = struct_cap.get(2).unwrap().as_str();

            let has_signer_attr = signer_attr_re.is_match(struct_body);
            let has_signer_type = signer_type_re
                .find_iter(struct_body)
                .any(|m| !m.as_str().trim_start().starts_with("//"));

            if has_signer_attr {
                println!(
                    "📦 Context: {:<30} in {:<40} ✅ via #[account(signer)]",
                    struct_name, file_name
                );
            } else if has_signer_type {
                println!(
                    "📦 Context: {:<30} in {:<40} ✅ via Signer<'info>",
                    struct_name, file_name
                );
            } else if ignored_files.contains(file_name.as_str()) {
                println!(
                    "⚠️  Context: {:<30} in {:<40} (IGNORED - no signer)",
                    struct_name, file_name
                );
            } else {
                println!(
                    "❌ Context: {:<30} in {:<40} ❗️ Missing signer",
                    struct_name, file_name
                );
                insecure_contexts.push((struct_name.to_string(), file_name.clone()));
            }
        }
    }

    if !insecure_contexts.is_empty() {
        println!("\n❌ Contexts missing signer verification:");
        for (ctx, file) in &insecure_contexts {
            println!("- {} (in {})", ctx, file);
        }
        panic!("❌ Some contexts are missing a signer or #[account(signer)].");
    }
}