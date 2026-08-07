use super::append_reply;
use crate::pixel::probe::MAX_REPLY_BYTES;

#[test]
fn reply_input_is_truncated_at_the_hard_cap() {
    let mut replies = vec![0x11; MAX_REPLY_BYTES - 3];
    assert!(append_reply(&mut replies, &[1, 2, 3, 4, 5]));
    assert_eq!(replies.len(), MAX_REPLY_BYTES);
    assert_eq!(&replies[MAX_REPLY_BYTES - 3..], &[1, 2, 3]);

    assert!(append_reply(&mut replies, &[6, 7, 8]));
    assert_eq!(replies.len(), MAX_REPLY_BYTES);
    assert_eq!(&replies[MAX_REPLY_BYTES - 3..], &[1, 2, 3]);
}
