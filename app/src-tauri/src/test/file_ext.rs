use screenshot_renamer::get_file_extension;

#[test]
fn test_file_ext(){


    let fake_image = "fake_image.png";
    let fake_image_path = std::path::Path::new(fake_image);

    assert_eq!("png", get_file_extension(fake_image_path));
    assert_ne!("jpeg", get_file_extension(fake_image_path));
    assert_ne!("jpg", get_file_extension(fake_image_path));

    

}