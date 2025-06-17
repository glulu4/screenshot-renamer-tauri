use std::fs::{self, File};
use std::path::PathBuf;

#[test]
fn test_rename_file() {
    // Create dummy file
    let original_path = PathBuf::from("temp_screenshot.png");
    File::create(&original_path).expect("Failed to create test file");

    // New name (without extension)
    let new_name = "renamed_test_file".to_string();

    // Call the function
    rename_file(&original_path, &new_name);

    // Check new file exists
    let renamed_path = PathBuf::from(format!("{}.png", new_name));
    assert!(renamed_path.exists(), "Renamed file should exist");

    // Cleanup
    fs::remove_file(renamed_path).expect("Failed to clean up test file");
}
