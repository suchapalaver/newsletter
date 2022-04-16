//! src/domain/subscriber_name.rs

use unicode_segmentation::UnicodeSegmentation;
#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
// [...]
}

impl AsRef<str> for SubscriberName {
// [...]
}

#[cfg(test)]
mod tests {
// [...]
}