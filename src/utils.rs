use std::borrow::Cow;

pub fn sanitize_game_name(name: &str) -> Cow<'_, str> {
    if name.contains(':') {
        // NTFS doesn't support : and this makes sense on Unix for cross-OS syncing
        Cow::Owned(name.replace(':', ""))
    } else {
        Cow::Borrowed(name)
    }
}
