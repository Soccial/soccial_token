// ============================================================================
// File: test_macros.rs
// Soccial Token – Logging Macros for Test Output
// ----------------------------------------------------------------------------
//
// This module provides standardized log macros for test output in the
// Soccial Token project. These macros help clearly mark key steps during
// integration testing with expressive emojis and consistent formatting.
//
// ----------------------------------------------------------------------------
// Included Macros:
// - `log_start!`   → Marks the beginning of a test phase (⏳)
// - `log_step!`    → Marks an intermediate progress step (🔄)
// - `log_done!`    → Marks successful completion of a step (✅)
// - `log_error!`   → Marks a failure or issue during testing (❌)
//
// ----------------------------------------------------------------------------
// Example Usage:
// log_start!("Initializing test context");
// log_step!("Creating accounts");
// log_done!("Test complete");
// log_error!("Unauthorized access");
//
// ----------------------------------------------------------------------------
// Benefits:
// ✔ Clear and expressive visual markers during test output
// ✔ Consistent formatting across all tests
// ✔ Helps debugging and understanding test flow
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================


/// Logs a standardized "starting" message with a ⏳ emoji.
/// Useful for marking the beginning of a logical test block or setup step.
///
/// # Example:

/// log_start!("Initializing test context");
/// // Output: ⏳  Initializing test context starting...

#[macro_export]
macro_rules! log_start {
    ($text:expr) => { anchor_lang::prelude::msg!("⏳  {} starting...", $text); };
}

/// Logs a standardized "step" message with a 🔄 emoji.
/// Useful for marking a step of a logical test block.
///
/// # Example:

/// log_step!("Step 2: Creating accounts");
/// // Output: 🔄  Initializing test context...

#[macro_export]
macro_rules! log_step {
    ($text:expr) => { anchor_lang::prelude::msg!("🔄  {}...", $text); };
}


/// Logs a standardized "error" message with a ❌ emoji.
/// Useful for marking the error step of a logical test block or setup step.
///
/// # Example:

/// log_error!("User is not the owner!");
/// // Output:  ❌  User is not the owner!

#[macro_export]
macro_rules! log_error {
    ($text:expr) => { anchor_lang::prelude::msg!("❌ ERROR:  {}", $text); };
}



/// Logs a standardized "done" message with a ✅ emoji.
/// Useful for marking the successful completion of a logical test block or setup step.
///
/// # Example:

/// log_done!("Initializing test context");
/// // Output: ✅  Initializing test context done!

#[macro_export]
macro_rules! log_done {
    ($text:expr) => { anchor_lang::prelude::msg!("✅  {} done!", $text); };
}
