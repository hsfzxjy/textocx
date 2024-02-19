use std::{borrow::Cow, iter::once, ops::Range};

pub(super) fn wrap_html<'a, I>(codes: I) -> String
where
    I: IntoIterator<Item = Cow<'a, str>>,
{
    let mut result = once(TEMPLATE_PREFIX.into())
        .chain(codes)
        .chain(once(TEMPLATE_SUFFIX.into()))
        .collect::<Vec<_>>()
        .concat();
    let code_len = result.len() - TEMPLATE_SUFFIX.len();
    let buf = unsafe { result.as_bytes_mut() };
    usize_to_str(&mut buf[META.start_html], META.html_open_in_prefix);
    usize_to_str(
        &mut buf[META.end_html],
        META.html_close_in_suffix + code_len,
    );
    usize_to_str(&mut buf[META.start_fragment], META.start_fragment_in_prefix);
    usize_to_str(
        &mut buf[META.end_fragment],
        META.end_fragment_in_suffix + code_len,
    );
    result
}
fn usize_to_str(buf: &mut [u8], mut x: usize) {
    let n = buf.len() as isize;
    let ori_x = x;
    let mut i = n - 1;
    while x > 0 && i >= 0 {
        let r = x % 10;
        x /= 10;
        buf[i as usize] = r as u8 + b'0';
        i -= 1;
    }
    if x > 0 {
        panic!("x too large: {}", ori_x)
    }
}

const TEMPLATE_PREFIX: &str = r#"Version:1.0
StartHTML:0000000000
EndHTML:0000000000
StartFragment:0000000000
EndFragment:0000000000
<html xmlns:v="urn:schemas-microsoft-com:vml"
xmlns:o="urn:schemas-microsoft-com:office:office"
xmlns:w="urn:schemas-microsoft-com:office:word"
xmlns:m="http://schemas.microsoft.com/office/2004/12/omml"
xmlns="http://www.w3.org/TR/REC-html40">
<head></head>
<body>
<!--StartFragment-->"#;
const TEMPLATE_SUFFIX: &str = r#"<!--EndFragment-->
</body>
</html>"#;

struct Meta {
    start_html: Range<usize>,
    end_html: Range<usize>,
    start_fragment: Range<usize>,
    end_fragment: Range<usize>,

    html_open_in_prefix: usize,
    html_close_in_suffix: usize,
    start_fragment_in_prefix: usize,
    end_fragment_in_suffix: usize,
}
const META: Meta = Meta::get();

impl Meta {
    const fn range_for(name: &'static str) -> Range<usize> {
        let idx = find(TEMPLATE_PREFIX, name) + name.len() + 1;
        idx..idx + 10
    }
    const fn get() -> Self {
        Meta {
            start_html: Self::range_for("StartHTML"),
            end_html: Self::range_for("EndHTML"),
            start_fragment: Self::range_for("StartFragment"),
            end_fragment: Self::range_for("EndFragment"),
            html_open_in_prefix: find(TEMPLATE_PREFIX, "<html"),
            html_close_in_suffix: find(TEMPLATE_SUFFIX, "</html"),
            start_fragment_in_prefix: find(TEMPLATE_PREFIX, "<!--StartFragment-->"),
            end_fragment_in_suffix: find(TEMPLATE_SUFFIX, "<!--EndFragment-->"),
        }
    }
}

/// A const function to find needle in s.
const fn find(s: &'static str, needle: &'static str) -> usize {
    let s = s.as_bytes();
    let needle = needle.as_bytes();
    let mut idx = 0usize;
    while idx < s.len() {
        let mut j = 0;
        let mut ok = true;
        while j < needle.len() {
            if needle[j] != s[idx + j] {
                ok = false;
                break;
            }
            j += 1;
        }
        if ok {
            return idx;
        }
        idx += 1;
    }
    panic!("cannot locate needle in s");
}
