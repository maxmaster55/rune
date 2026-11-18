use super::*;

#[test]
fn a_file_is_keyed_by_its_family_stem() {
    let key = |p: &str| family_key(Path::new(p));
    assert_eq!(key("/system/fonts/NotoSansArabic-Bold.ttf").as_deref(), Some("notosansarabic"));
    assert_eq!(key("/system/fonts/NotoColorEmoji.ttf").as_deref(), Some("notocoloremoji"));
    assert_eq!(key("/system/fonts/NotoSansCJK-Regular.ttc").as_deref(), Some("notosanscjk"));
    assert_eq!(key("/system/fonts/NotoSansSymbols-Regular-Subsetted.ttf").as_deref(), Some("notosanssymbols"));
    assert_eq!(key("/product/fonts/DroidSansMono.ttf").as_deref(), Some("droidsansmono"));
    assert_eq!(key("/system/fonts/-Bold.ttf"), None);
}

#[test]
fn a_family_name_matches_its_files_and_no_longer_family() {
    assert_eq!(name_key("Noto Sans Arabic"), "notosansarabic");
    assert_eq!(name_key("Noto Color Emoji"), "notocoloremoji");
    let files: Vec<(String, PathBuf)> = [
        "/system/fonts/NotoNaskhArabic-Regular.ttf",
        "/system/fonts/NotoNaskhArabicUI-Regular.ttf",
        "/system/fonts/NotoNaskhArabic-Bold.ttf",
        "/system/fonts/NotoColorEmoji.ttf",
        "/system/fonts/NotoColorEmojiFlags.ttf",
    ]
    .into_iter()
    .map(|p| (family_key(Path::new(p)).unwrap(), PathBuf::from(p)))
    .collect();
    let mut out = Vec::new();
    files_of_key(&files, &name_key("Noto Naskh Arabic"), &mut out);
    // The whole family, Bold too, and not the UI variant whose name only starts the same.
    assert_eq!(out, [PathBuf::from("/system/fonts/NotoNaskhArabic-Regular.ttf"), PathBuf::from("/system/fonts/NotoNaskhArabic-Bold.ttf")]);
    // A file already found is not added twice.
    files_of_key(&files, "notonaskharabic", &mut out);
    assert_eq!(out.len(), 2);
    files_of_key(&files, &name_key("Noto Color Emoji"), &mut out);
    assert_eq!(out.last(), Some(&PathBuf::from("/system/fonts/NotoColorEmoji.ttf")));
    assert_eq!(out.len(), 3);
}
