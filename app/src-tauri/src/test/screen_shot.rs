

#[test]
fn test_is_new_screenshot() {
    use std::path::PathBuf;

    let valid_existing = PathBuf::from("tests/assets/Screenshot_2025-06-12.png");
    let nonexistent = PathBuf::from("/some/clearly/fake/Screenshot_does_not_exist.png");
    let wrong_ext = PathBuf::from("tests/assets/screenshot.txt");
    let wrong_name = PathBuf::from("tests/assets/image_01.png");




    // Create dummy file for valid_existing if needed
    std::fs::create_dir_all("tests/assets").unwrap();
    std::fs::write(&valid_existing, b"test").unwrap();

    assert!(is_new_screenshot(&valid_existing), "Should detect real screenshot");
    assert!(!is_new_screenshot(&nonexistent), "Should skip nonexistent file");
    assert!(!is_new_screenshot(&wrong_ext), "Wrong extension should return false");
    assert!(!is_new_screenshot(&wrong_name), "Wrong name should return false");

    // Clean up
    std::fs::remove_file(&valid_existing).unwrap();
}
