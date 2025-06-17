use screenshot_renamer::encode_image_to_base64;




#[test]
fn test_encode_image_to_base64() {
    use std::path::PathBuf;

    let valid_image = PathBuf::from("tests/assets/Screenshot_2025-06-12.png");
    let nonexistent_image = PathBuf::from("/some/clearly/fake/Screenshot_does_not_exist.png");

    // Create dummy file for valid_image if needed
    std::fs::create_dir_all("tests/assets").unwrap();
    std::fs::write(&valid_image, b"test image content").unwrap();

    let base64_string = encode_image_to_base64(&valid_image);
    assert!(!base64_string.is_empty(), "Base64 string should not be empty");

    let empty_base64_string = encode_image_to_base64(&nonexistent_image);
    assert!(empty_base64_string.is_empty(), "Base64 string for nonexistent file should be empty");

    // Clean up
    std::fs::remove_file(&valid_image).unwrap();
}