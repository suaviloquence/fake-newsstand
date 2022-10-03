use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BlogInfo {
	uuid: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TextSubtype {
	/// intended for post headings
	Heading1,
	/// intended for section subheadings
	Heading2,
	/// Tumblr Official clients display this with a large cursive font.
	Quirky,
	/// Intended for short quotations, official Tumblr clients display this with a large serif font.
	Quote,
	/// Intended for longer quotations or photo captions, official Tumblr clients indent this text block.
	Indented,
	/// Intended to mimic the behavior of the Chat Post type, official Tumblr clients display this with a monospace font.
	Chat,
	/// Intended to be an ordered list item prefixed by a number, see [ContentBlock::Text].indent_level
	OrderedListItem,
	/// Intended to be an unordered list item prefixed with a bullet, see [ContentBlock::Text].indent_level
	UnorderedListItem,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PostInfo {
	id: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TextFormatType {
	Bold,
	Italic,
	Strikethrough,
	Small,
	Link {
		url: String,
	},
	Mention {
		blog: BlogInfo,
	},
	Color {
		/// include leading '#'
		hex: String,
	},
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TextFormatting {
	/// indexed by chars, not bytes
	start: usize,
	end: usize,
	#[serde(flatten)]
	format_type: TextFormatType,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Media {
	url: String,
	#[serde(rename = "type")]
	mime_type: Option<String>,
	width: Option<u64>,
	height: Option<u64>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Attribution {
	Post {
		url: String,
		blog: BlogInfo,
		post: PostInfo,
	},
	Link {
		url: String,
	},
	Blog {
		blog: BlogInfo,
	},
	App {
		url: String,
		app_name: Option<String>,
		display_text: Option<String>,
		logo: Option<Media>,
	},
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum MediaSource {
	Url { url: String },
	Media { media: Media },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct EmbedIframe {
	url: String,
	width: u64,
	height: u64,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaywallSubtype {
	Cta { title: String },
	Disabled { title: String },
	Divider { color: Option<String> },
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
	Text {
		text: String,
		subtype: Option<TextSubtype>,
		/// must be in range 0..=7
		indent_level: Option<u8>,
		formatting: Option<Vec<TextFormatting>>,
	},
	Image {
		media: Vec<Media>,
		// TODO colors: ?
		feedback_token: Option<String>,
		/// for GIFs
		poster: Option<Media>,
		attribution: Option<Attribution>,
		alt_text: Option<String>,
		caption: Option<String>,
	},
	Link {
		url: String,
		title: Option<String>,
		description: Option<String>,
		author: Option<String>,
		site_name: Option<String>,
		/// ignored on create, sent on retrieve
		display_url: Option<String>,
		poster: Option<Media>,
	},
	Audio {
		source: MediaSource,
		// TODO provider: tumblr, soundcloud, etc
		title: Option<String>,
		artist: Option<String>,
		album: Option<String>,
		poster: Option<Media>,
		embed_html: Option<String>,
		embed_url: Option<String>,
		// TODO metadata: Option<provider specific metadata object>
		attribution: Option<Attribution>,
	},
	Video {
		source: MediaSource,
		// TODO provider: tumblr, youtube, etc.
		embed_html: Option<String>,
		embed_iframe: Option<EmbedIframe>,
		embed_url: Option<String>,
		poster: Option<Media>,
		// TODO metadata: Option<provider specific metadata object>
		attribution: Option<Attribution>,
		can_autoplay_on_cellular: Option<bool>,
	},
	Paywall {
		#[serde(flatten)]
		subtype: PaywallSubtype,
		url: String,
		text: String,
		is_visible: Option<bool>,
	},
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RowDisplayMode {
	Carousel,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RowDisplay {
	blocks: Vec<u64>,
	mode: Option<RowDisplayMode>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum LayoutBlock {
	Rows {
		blocks: Vec<RowDisplay>,
		truncate_after: Option<u64>,
	},
	/// legacy.  Either truncate_after or blocks is required
	Condensed {
		truncate_after: Option<u64>,
		/// must start with 0 and be sequential
		blocks: Option<Vec<u64>>,
	},
	Ask {
		/// which block indices are part of the ask portion of the post
		blocks: Vec<u64>,
		/// if None, ask is anonymous.  Otherwise, (should be) guaranteed to be Attribution::Blog
		attribution: Option<Attribution>,
	},
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum ReblogTrail {
	Ok {
		post: PostInfo,
		blog: BlogInfo,
		content: Vec<ContentBlock>,
		layout: Vec<LayoutBlock>,
	},
	Broken {
		broken_blog_name: String,
		content: Vec<ContentBlock>,
		layout: Vec<LayoutBlock>,
	},
}

#[derive(Serialize)]
pub struct Post {
	pub id: u64,
}

#[cfg(test)]
mod tests {
	use pretty_assertions::assert_eq;
	use serde_json::{from_str, to_string_pretty};

	use super::*;

	macro_rules! s {
		($str: expr) => {
			$str.to_owned()
		};
	}

	fn assert_serde<'de, T: Serialize + Deserialize<'de> + std::fmt::Debug + PartialEq>(
		str: &'static str,
		obj: T,
	) {
		assert_eq!(from_str::<'de, T>(str).unwrap(), obj);
		assert_eq!(to_string_pretty(&obj).unwrap(), str);
	}

	#[test]
	fn test_text_blocks() {
		assert_eq!(
			from_str::<ContentBlock>(
				r#"{
				"type": "text",
				"text": "Hello world!"
			}"#,
			)
			.unwrap(),
			ContentBlock::Text {
				text: "Hello world!".to_owned(),
				subtype: None,
				indent_level: None,
				formatting: None
			}
		);

		assert_eq!(
			from_str::<Vec<ContentBlock>>(
				r#"[
			{
					"type": "text",
					"subtype": "heading1",
					"text": "Sward's Shopping List"
			},
			{
					"type": "text",
					"subtype": "ordered-list-item",
					"text": "First level: Fruit"
			},
			{
					"type": "text",
					"subtype": "unordered-list-item",
					"text": "Second level: Apples",
					"indent_level": 1
			},
			{
					"type": "text",
					"subtype": "ordered-list-item",
					"text": "Third Level: Green",
					"indent_level": 2
			},
			{
					"type": "text",
					"subtype": "unordered-list-item",
					"text": "Second level: Pears",
					"indent_level": 1
			},
			{
					"type": "text",
					"subtype": "ordered-list-item",
					"text": "First level: Pears"
			}
	]"#
			)
			.unwrap(),
			vec![
				ContentBlock::Text {
					text: "Sward's Shopping List".to_owned(),
					subtype: Some(TextSubtype::Heading1),
					indent_level: None,
					formatting: None
				},
				ContentBlock::Text {
					text: "First level: Fruit".to_owned(),
					subtype: Some(TextSubtype::OrderedListItem),
					indent_level: None,
					formatting: None
				},
				ContentBlock::Text {
					text: "Second level: Apples".to_owned(),
					subtype: Some(TextSubtype::UnorderedListItem),
					indent_level: Some(1),
					formatting: None
				},
				ContentBlock::Text {
					text: "Third Level: Green".to_owned(),
					subtype: Some(TextSubtype::OrderedListItem),
					indent_level: Some(2),
					formatting: None
				},
				ContentBlock::Text {
					text: "Second level: Pears".to_owned(),
					subtype: Some(TextSubtype::UnorderedListItem),
					indent_level: Some(1),
					formatting: None
				},
				ContentBlock::Text {
					text: "First level: Pears".to_owned(),
					subtype: Some(TextSubtype::OrderedListItem),
					indent_level: None,
					formatting: None
				}
			]
		);

		assert_eq!(
			from_str::<ContentBlock>(
				r#"{
			"type": "text",
			"text": "some bold and italic text",
			"formatting": [
					{
							"start": 5,
							"end": 9,
							"type": "bold"
					},
					{
							"start": 14,
							"end": 20,
							"type": "italic"
					}
			]
	}"#
			)
			.unwrap(),
			ContentBlock::Text {
				text: "some bold and italic text".to_owned(),
				subtype: None,
				indent_level: None,
				formatting: Some(vec![
					TextFormatting {
						start: 5,
						end: 9,
						format_type: TextFormatType::Bold
					},
					TextFormatting {
						start: 14,
						end: 20,
						format_type: TextFormatType::Italic
					}
				])
			}
		);

		assert_eq!(
			from_str::<ContentBlock>(
				r#"{
			"type": "text",
			"text": "Shout out to @david",
			"formatting": [
					{
							"start": 13,
							"end": 19,
							"type": "mention",
							"blog": {
									"uuid": "t:123456abcdf",
									"name": "david",
									"url": "https://davidslog.com/"
							}
					}
			]
	}"#
			)
			.unwrap(),
			ContentBlock::Text {
				text: "Shout out to @david".to_owned(),
				subtype: None,
				indent_level: None,
				formatting: Some(vec![TextFormatting {
					start: 13,
					end: 19,
					format_type: TextFormatType::Mention {
						blog: BlogInfo {
							uuid: "t:123456abcdf".to_owned()
						}
					}
				}])
			}
		);
	}

	#[test]
	fn test_examples_from_api_spec() {
		assert_serde(
			r#"{
  "type": "ask",
  "blocks": [
    0,
    1
  ],
  "attribution": {
    "type": "blog",
    "blog": {
      "uuid": "abcdef"
    }
  }
}"#,
			LayoutBlock::Ask {
				blocks: vec![0, 1],
				attribution: Some(Attribution::Blog {
					blog: BlogInfo { uuid: s!("abcdef") },
				}),
			},
		);

		assert_serde(
			r#"[
  {
    "broken_blog_name": "old-broken-blog",
    "content": [
      {
        "type": "text",
        "text": "this is the root Post, which is broken"
      }
    ],
    "layout": []
  },
  {
    "broken_blog_name": "another-broken-blog",
    "content": [
      {
        "type": "text",
        "text": "this is the parent Post, which is also broken"
      },
      {
        "type": "text",
        "text": "this is another text block in the broken parent Post"
      }
    ],
    "layout": []
  }
]"#,
			vec![
				ReblogTrail::Broken {
					broken_blog_name: s!("old-broken-blog"),
					content: vec![ContentBlock::Text {
						text: s!("this is the root Post, which is broken"),
						subtype: None,
						indent_level: None,
						formatting: None,
					}],
					layout: vec![],
				},
				ReblogTrail::Broken {
					broken_blog_name: s!("another-broken-blog"),
					content: vec![
						ContentBlock::Text {
							text: s!("this is the parent Post, which is also broken"),
							subtype: None,
							indent_level: None,
							formatting: None,
						},
						ContentBlock::Text {
							text: s!("this is another text block in the broken parent Post"),
							subtype: None,
							indent_level: None,
							formatting: None,
						},
					],
					layout: vec![],
				},
			],
		);

		assert_serde(r#"{
  "type": "post",
  "url": "http://www.davidslog.com/153957802620/five-years-of-working-with-this-awesome-girl",
  "blog": {
    "uuid": "t:123456abcdf"
  },
  "post": {
    "id": 1234567890
  }
}"#,
			 Attribution::Post {
				 url: s!("http://www.davidslog.com/153957802620/five-years-of-working-with-this-awesome-girl"),
				 blog: BlogInfo { uuid: s!("t:123456abcdf") },
				 post: PostInfo { id: 1234567890 }
				}
			);

		assert_serde(
			r#"{
  "type": "link",
  "url": "http://shahkashani.com"
}"#,
			Attribution::Link {
				url: s!("http://shahkashani.com"),
			},
		);

		assert_serde(
			r#"{
  "type": "app",
  "url": "https://www.instagram.com/p/BVZyxTklQWX/",
  "app_name": "Instagram",
  "display_text": "tibbythecorgi - Very Cute",
  "logo": {
    "url": "https://scontent.cdninstagram.com/path/to/logo.jpg",
    "type": "image/jpeg",
    "width": 64,
    "height": 64
  }
}"#,
			Attribution::App {
				url: "https://www.instagram.com/p/BVZyxTklQWX/".to_owned(),
				app_name: Some("Instagram".to_owned()),
				display_text: Some("tibbythecorgi - Very Cute".to_owned()),
				logo: Some(Media {
					url: s!("https://scontent.cdninstagram.com/path/to/logo.jpg"),
					mime_type: Some(s!("image/jpeg")),
					width: Some(64),
					height: Some(64),
				}),
			},
		);
	}
}
