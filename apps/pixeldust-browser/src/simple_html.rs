use eframe::egui;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;
use url::Url;

#[derive(Debug, Clone)]
pub struct HtmlDocument {
    pub root: HtmlElement,
    pub title: Option<String>,
    styles: StyleSheet,
}

#[derive(Debug, Clone)]
pub struct HtmlElement {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<HtmlNode>,
}

#[derive(Debug, Clone)]
pub enum HtmlNode {
    Element(HtmlElement),
    Text(String),
}

#[derive(Debug)]
enum Token {
    Start {
        name: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    },
    End {
        name: String,
    },
    Text(String),
}

#[derive(Debug, Default)]
pub struct RenderAction {
    pub navigate_to: Option<String>,
    pub dom_events: Vec<DomEventRequest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomEventKind {
    Click,
    Input,
    Submit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomEventRequest {
    pub kind: DomEventKind,
    pub target_id: Option<String>,
    pub inline_handler: String,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderImage {
    pub texture_id: egui::TextureId,
    pub size: egui::Vec2,
}

#[derive(Debug)]
pub struct RenderResources<'a> {
    pub images: &'a HashMap<String, RenderImage>,
}

#[derive(Debug, Clone, Default)]
pub struct SubresourceManifest {
    pub stylesheets: Vec<String>,
    pub images: Vec<String>,
    #[allow(dead_code)]
    pub scripts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptDescriptor {
    External { url: String },
    Inline { source: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdElementSnapshot {
    pub id: String,
    pub tag_name: String,
    pub text_content: String,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
struct StyleSheet {
    rules: Vec<CssRule>,
}

#[derive(Debug, Clone)]
struct CssRule {
    sel: Selector,
    specificity: u16,
    declarations: Vec<CssDeclaration>,
}

#[derive(Debug, Clone)]
struct CssDeclaration {
    name: String,
    value: String,
    important: bool,
    source_order: usize,
    parsed: StyleProps,
}

#[derive(Debug, Clone, Default)]
struct Selector {
    segments: Vec<SelectorSegment>,
}

#[derive(Debug, Clone)]
struct SelectorSegment {
    simple: SimpleSelector,
    combinator_to_next: Option<SelectorCombinator>,
}

#[derive(Debug, Clone, Default)]
struct SimpleSelector {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectorCombinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone)]
struct SelectorSubject {
    tag: String,
    id: Option<String>,
    classes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Display {
    Block,
    Inline,
    Flex,
    Grid,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlignContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlexWrap {
    NoWrap,
    Wrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionMode {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverflowMode {
    Visible,
    Hidden,
    Auto,
    Scroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WhiteSpaceMode {
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FontFamilyChoice {
    Proportional,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptPosition {
    Baseline,
    Sub,
    Sup,
}

#[derive(Debug, Clone, Copy, Default)]
struct TextEffects {
    strong: bool,
    italics: bool,
    mono: bool,
    underline: bool,
    strike: bool,
    mark: bool,
    small: bool,
    script: Option<ScriptPosition>,
}

// Sourced from MDN HTML element reference:
// https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements
#[cfg_attr(not(test), allow(dead_code))]
const MDN_REFERENCE_ELEMENTS: &[&str] = &[
    "a",
    "abbr",
    "acronym",
    "address",
    "applet",
    "area",
    "article",
    "aside",
    "audio",
    "b",
    "base",
    "basefont",
    "bdi",
    "bdo",
    "bgsound",
    "big",
    "blink",
    "blockquote",
    "body",
    "br",
    "button",
    "canvas",
    "caption",
    "center",
    "cite",
    "code",
    "col",
    "colgroup",
    "content",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "em",
    "embed",
    "fencedframe",
    "fieldset",
    "figcaption",
    "figure",
    "font",
    "footer",
    "form",
    "frame",
    "frameset",
    "geolocation",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "i",
    "iframe",
    "image",
    "img",
    "input",
    "ins",
    "kbd",
    "keygen",
    "label",
    "legend",
    "li",
    "link",
    "main",
    "map",
    "mark",
    "marquee",
    "math",
    "menu",
    "menuitem",
    "meta",
    "meter",
    "multicol",
    "nav",
    "nextid",
    "nobr",
    "noembed",
    "noframes",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "output",
    "p",
    "param",
    "picture",
    "plaintext",
    "portal",
    "pre",
    "progress",
    "q",
    "rb",
    "rp",
    "rt",
    "rtc",
    "ruby",
    "s",
    "samp",
    "script",
    "search",
    "section",
    "select",
    "selectedcontent",
    "shadow",
    "slot",
    "small",
    "source",
    "spacer",
    "span",
    "strike",
    "strong",
    "style",
    "sub",
    "summary",
    "sup",
    "svg",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "title",
    "tr",
    "track",
    "tt",
    "u",
    "ul",
    "var",
    "video",
    "wbr",
    "xmp",
];

// Sourced from MDN HTML attribute reference:
// https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Attributes
#[cfg_attr(not(test), allow(dead_code))]
const MDN_REFERENCE_ATTRIBUTES: &[&str] = &[
    "accept",
    "accept-charset",
    "accesskey",
    "action",
    "align",
    "allow",
    "alpha",
    "alt",
    "as",
    "async",
    "autocapitalize",
    "autocomplete",
    "autoplay",
    "background",
    "bgcolor",
    "border",
    "capture",
    "charset",
    "checked",
    "cite",
    "class",
    "color",
    "colorspace",
    "cols",
    "colspan",
    "content",
    "contenteditable",
    "controls",
    "coords",
    "crossorigin",
    "csp",
    "data",
    "data-*",
    "datetime",
    "decoding",
    "default",
    "defer",
    "dir",
    "dirname",
    "disabled",
    "download",
    "draggable",
    "elementtiming",
    "enctype",
    "enterkeyhint",
    "fetchpriority",
    "for",
    "form",
    "formaction",
    "formenctype",
    "formmethod",
    "formnovalidate",
    "formtarget",
    "headers",
    "height",
    "hidden",
    "high",
    "href",
    "hreflang",
    "http-equiv",
    "id",
    "integrity",
    "ismap",
    "itemprop",
    "kind",
    "label",
    "lang",
    "language",
    "list",
    "loading",
    "loop",
    "low",
    "max",
    "maxlength",
    "media",
    "method",
    "min",
    "minlength",
    "multiple",
    "muted",
    "name",
    "novalidate",
    "open",
    "optimum",
    "pattern",
    "ping",
    "placeholder",
    "playsinline",
    "poster",
    "preload",
    "readonly",
    "referrerpolicy",
    "rel",
    "required",
    "reversed",
    "role",
    "rows",
    "rowspan",
    "sandbox",
    "scope",
    "selected",
    "shape",
    "size",
    "sizes",
    "slot",
    "span",
    "spellcheck",
    "src",
    "srcdoc",
    "srclang",
    "srcset",
    "start",
    "step",
    "style",
    "summary",
    "tabindex",
    "target",
    "title",
    "translate",
    "type",
    "usemap",
    "value",
    "width",
    "wrap",
];

const MDN_REFERENCE_CSS_PROPERTIES_RAW: &str = include_str!("mdn_css_reference_properties.txt");

fn canonical_element_tag(tag: &str) -> &str {
    match tag {
        "image" => "img",
        _ => tag,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn is_mdn_reference_element(tag: &str) -> bool {
    let tag = canonical_element_tag(tag);
    MDN_REFERENCE_ELEMENTS
        .iter()
        .any(|candidate| *candidate == tag)
}

#[cfg_attr(not(test), allow(dead_code))]
fn is_mdn_reference_attribute(name: &str) -> bool {
    let lower = name.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return false;
    }

    if lower.starts_with("data-") && lower.len() > "data-".len() {
        return true;
    }
    if lower.starts_with("aria-") && lower.len() > "aria-".len() {
        return true;
    }
    if lower.starts_with("on") && lower.len() > 2 {
        return true;
    }

    MDN_REFERENCE_ATTRIBUTES
        .iter()
        .any(|candidate| *candidate == lower)
}

fn mdn_reference_css_properties() -> &'static [&'static str] {
    static PROPERTIES: OnceLock<Vec<&'static str>> = OnceLock::new();
    PROPERTIES
        .get_or_init(|| {
            MDN_REFERENCE_CSS_PROPERTIES_RAW
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .filter(|line| !line.starts_with('#'))
                .filter(|line| *line != "--*")
                .collect::<Vec<_>>()
        })
        .as_slice()
}

fn mdn_reference_css_property_set() -> &'static HashSet<&'static str> {
    static PROPERTY_SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    PROPERTY_SET.get_or_init(|| {
        mdn_reference_css_properties()
            .iter()
            .copied()
            .collect::<HashSet<_>>()
    })
}

#[cfg_attr(not(test), allow(dead_code))]
fn is_mdn_reference_css_property(name: &str) -> bool {
    let lower = name.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return false;
    }

    // Custom properties and vendor-prefixed properties are valid CSS extensions.
    if (lower.starts_with("--") && lower.len() > 2) || (lower.starts_with('-') && lower.len() > 1)
    {
        return true;
    }

    mdn_reference_css_property_set().contains(lower.as_str())
}

fn is_non_rendered_element_tag(tag: &str) -> bool {
    matches!(
        canonical_element_tag(tag),
        "base"
            | "head"
            | "link"
            | "meta"
            | "nextid"
            | "noscript"
            | "script"
            | "style"
            | "template"
            | "title"
    )
}

fn is_static_fallback_ignored_tag(tag: &str) -> bool {
    matches!(
        canonical_element_tag(tag),
        "base" | "head" | "link" | "meta" | "nextid" | "script" | "style" | "template" | "title"
    )
}

#[derive(Debug, Clone, Copy, Default)]
struct Edges {
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    left: Option<f32>,
}

impl Edges {
    fn apply(&mut self, other: &Edges) {
        if other.top.is_some() {
            self.top = other.top;
        }
        if other.right.is_some() {
            self.right = other.right;
        }
        if other.bottom.is_some() {
            self.bottom = other.bottom;
        }
        if other.left.is_some() {
            self.left = other.left;
        }
    }

    fn all(value: f32) -> Self {
        Self {
            top: Some(value),
            right: Some(value),
            bottom: Some(value),
            left: Some(value),
        }
    }

    fn top_or(self, default: f32) -> f32 {
        self.top.unwrap_or(default)
    }

    fn right_or(self, default: f32) -> f32 {
        self.right.unwrap_or(default)
    }

    fn bottom_or(self, default: f32) -> f32 {
        self.bottom.unwrap_or(default)
    }

    fn left_or(self, default: f32) -> f32 {
        self.left.unwrap_or(default)
    }

    fn max_or(self, default: f32) -> f32 {
        self.top_or(default)
            .max(self.right_or(default))
            .max(self.bottom_or(default))
            .max(self.left_or(default))
    }

    fn non_negative(self) -> Self {
        Self {
            top: self.top.map(|value| value.max(0.0)),
            right: self.right.map(|value| value.max(0.0)),
            bottom: self.bottom.map(|value| value.max(0.0)),
            left: self.left.map(|value| value.max(0.0)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum EdgeSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Default)]
struct StyleProps {
    display: Option<Display>,
    visibility_hidden: Option<bool>,
    opacity: Option<f32>,
    text_align: Option<TextAlign>,
    font_family: Option<FontFamilyChoice>,
    color: Option<egui::Color32>,
    bg: Option<egui::Color32>,
    font_size: Option<f32>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strike: Option<bool>,
    script: Option<ScriptPosition>,
    line_height: Option<f32>,
    flex_direction: Option<FlexDirection>,
    flex_wrap: Option<FlexWrap>,
    justify_content: Option<JustifyContent>,
    align_items: Option<AlignItems>,
    align_content: Option<AlignContent>,
    gap: Option<f32>,
    text_transform: Option<TextTransform>,
    white_space: Option<WhiteSpaceMode>,
    list_style_type: Option<String>,
    position: Option<PositionMode>,
    inset_top: Option<f32>,
    inset_right: Option<f32>,
    inset_bottom: Option<f32>,
    inset_left: Option<f32>,
    z_index: Option<i32>,
    overflow_x: Option<OverflowMode>,
    overflow_y: Option<OverflowMode>,
    width: Option<f32>,
    width_percent: Option<f32>,
    min_width: Option<f32>,
    max_width: Option<f32>,
    height: Option<f32>,
    min_height: Option<f32>,
    max_height: Option<f32>,
    margin: Edges,
    padding: Edges,
    border_width: Edges,
    border_color: Option<egui::Color32>,
    border_radius: Option<f32>,
    raw_css: HashMap<String, String>,
}

impl StyleProps {
    fn is_empty(&self) -> bool {
        self.display.is_none()
            && self.visibility_hidden.is_none()
            && self.opacity.is_none()
            && self.text_align.is_none()
            && self.font_family.is_none()
            && self.color.is_none()
            && self.bg.is_none()
            && self.font_size.is_none()
            && self.bold.is_none()
            && self.italic.is_none()
            && self.underline.is_none()
            && self.strike.is_none()
            && self.script.is_none()
            && self.line_height.is_none()
            && self.flex_direction.is_none()
            && self.flex_wrap.is_none()
            && self.justify_content.is_none()
            && self.align_items.is_none()
            && self.align_content.is_none()
            && self.gap.is_none()
            && self.text_transform.is_none()
            && self.white_space.is_none()
            && self.list_style_type.is_none()
            && self.position.is_none()
            && self.inset_top.is_none()
            && self.inset_right.is_none()
            && self.inset_bottom.is_none()
            && self.inset_left.is_none()
            && self.z_index.is_none()
            && self.overflow_x.is_none()
            && self.overflow_y.is_none()
            && self.width.is_none()
            && self.width_percent.is_none()
            && self.min_width.is_none()
            && self.max_width.is_none()
            && self.height.is_none()
            && self.min_height.is_none()
            && self.max_height.is_none()
            && self.margin.top.is_none()
            && self.margin.right.is_none()
            && self.margin.bottom.is_none()
            && self.margin.left.is_none()
            && self.padding.top.is_none()
            && self.padding.right.is_none()
            && self.padding.bottom.is_none()
            && self.padding.left.is_none()
            && self.border_width.top.is_none()
            && self.border_width.right.is_none()
            && self.border_width.bottom.is_none()
            && self.border_width.left.is_none()
            && self.border_color.is_none()
            && self.border_radius.is_none()
            && self.raw_css.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CascadePriority {
    important: bool,
    specificity: u16,
    source_order: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct EdgePriority {
    top: Option<CascadePriority>,
    right: Option<CascadePriority>,
    bottom: Option<CascadePriority>,
    left: Option<CascadePriority>,
}

#[derive(Debug, Clone, Default)]
struct StylePriority {
    display: Option<CascadePriority>,
    visibility_hidden: Option<CascadePriority>,
    opacity: Option<CascadePriority>,
    text_align: Option<CascadePriority>,
    font_family: Option<CascadePriority>,
    color: Option<CascadePriority>,
    bg: Option<CascadePriority>,
    font_size: Option<CascadePriority>,
    bold: Option<CascadePriority>,
    italic: Option<CascadePriority>,
    underline: Option<CascadePriority>,
    strike: Option<CascadePriority>,
    script: Option<CascadePriority>,
    line_height: Option<CascadePriority>,
    flex_direction: Option<CascadePriority>,
    flex_wrap: Option<CascadePriority>,
    justify_content: Option<CascadePriority>,
    align_items: Option<CascadePriority>,
    align_content: Option<CascadePriority>,
    gap: Option<CascadePriority>,
    text_transform: Option<CascadePriority>,
    white_space: Option<CascadePriority>,
    list_style_type: Option<CascadePriority>,
    position: Option<CascadePriority>,
    inset_top: Option<CascadePriority>,
    inset_right: Option<CascadePriority>,
    inset_bottom: Option<CascadePriority>,
    inset_left: Option<CascadePriority>,
    z_index: Option<CascadePriority>,
    overflow_x: Option<CascadePriority>,
    overflow_y: Option<CascadePriority>,
    width: Option<CascadePriority>,
    width_percent: Option<CascadePriority>,
    min_width: Option<CascadePriority>,
    max_width: Option<CascadePriority>,
    height: Option<CascadePriority>,
    min_height: Option<CascadePriority>,
    max_height: Option<CascadePriority>,
    margin: EdgePriority,
    padding: EdgePriority,
    border_width: EdgePriority,
    border_color: Option<CascadePriority>,
    border_radius: Option<CascadePriority>,
    raw_css: HashMap<String, CascadePriority>,
}

struct Ctx<'a> {
    base_url: &'a str,
    styles: &'a StyleSheet,
    resources: &'a RenderResources<'a>,
    action: &'a mut RenderAction,
    form_state: &'a mut HashMap<String, String>,
    form_stack: Vec<FormRuntime>,
    form_fields: HashMap<String, HashMap<String, String>>,
    ancestor_stack: Vec<SelectorSubject>,
}

#[derive(Debug, Clone)]
struct FormRuntime {
    key: String,
    action_url: String,
    method: String,
    form_id: Option<String>,
    onsubmit: Option<String>,
}

impl HtmlDocument {
    pub fn parse(source: &str) -> Self {
        let tokens = tokenize(source);
        let root = build_tree(tokens);
        let styles = extract_styles(&root);
        let title = find_title(&root);
        Self {
            root,
            title,
            styles,
        }
    }

    pub fn append_stylesheet_source(&mut self, source: &str) {
        self.styles.rules.extend(parse_css_rules(source));
    }

    pub fn collect_subresources(&self, base_url: &str) -> SubresourceManifest {
        let mut stylesheets = HashSet::new();
        let mut images = HashSet::new();
        let mut scripts = HashSet::new();

        collect_subresources_from_nodes(
            &self.root.children,
            base_url,
            &mut stylesheets,
            &mut images,
            &mut scripts,
        );

        let mut stylesheets = stylesheets.into_iter().collect::<Vec<_>>();
        let mut images = images.into_iter().collect::<Vec<_>>();
        let mut scripts = scripts.into_iter().collect::<Vec<_>>();

        stylesheets.sort();
        images.sort();
        scripts.sort();

        SubresourceManifest {
            stylesheets,
            images,
            scripts,
        }
    }

    pub fn css_rule_count(&self) -> usize {
        self.styles.rules.len()
    }

    pub fn inline_style_tag_count(&self) -> usize {
        count_style_tags(&self.root.children)
    }

    pub fn collect_script_descriptors(&self, base_url: &str) -> Vec<ScriptDescriptor> {
        let mut scripts = Vec::new();
        collect_script_descriptors(&self.root.children, base_url, &mut scripts);
        scripts
    }

    pub fn collect_id_elements(&self, max_elements: usize) -> Vec<IdElementSnapshot> {
        let mut out = Vec::new();
        if max_elements == 0 {
            return out;
        }
        collect_id_elements(&self.root.children, max_elements, &mut out);
        out
    }

    #[cfg(test)]
    pub fn visible_text_len(&self) -> usize {
        let text = if let Some(body) = find_first_element(&self.root.children, "body") {
            collect_text(&body.children)
        } else {
            collect_text(&self.root.children)
        };
        collapse_whitespace(&text).len()
    }

    pub fn renderable_text_len(&self) -> usize {
        let mut out = String::new();
        let inherited = StyleProps::default();
        let mut ancestors = Vec::new();
        if let Some(body) = find_first_element(&self.root.children, "body") {
            ancestors.push(selector_subject(body));
            collect_renderable_text(
                &body.children,
                &self.styles,
                &inherited,
                &mut ancestors,
                &mut out,
            );
        } else {
            collect_renderable_text(
                &self.root.children,
                &self.styles,
                &inherited,
                &mut ancestors,
                &mut out,
            );
        }
        collapse_whitespace(&out).len()
    }

    pub fn static_text_fallback(&self, max_chars: usize) -> String {
        if max_chars == 0 {
            return String::new();
        }

        let mut out = String::new();
        collect_static_fallback_text(&self.root.children, &mut out);
        let collapsed = collapse_whitespace(&out);
        if collapsed.chars().count() <= max_chars {
            return collapsed;
        }

        collapsed.chars().take(max_chars).collect()
    }
}

pub fn render_document(
    ui: &mut egui::Ui,
    doc: &HtmlDocument,
    base_url: &str,
    resources: &RenderResources<'_>,
    action: &mut RenderAction,
    form_state: &mut HashMap<String, String>,
) {
    // Match browser defaults regardless of host app theme.
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, egui::Color32::WHITE);

    let mut ctx = Ctx {
        base_url,
        styles: &doc.styles,
        resources,
        action,
        form_state,
        form_stack: Vec::new(),
        form_fields: HashMap::new(),
        ancestor_stack: Vec::new(),
    };
    let inherited = StyleProps {
        color: Some(egui::Color32::BLACK),
        ..StyleProps::default()
    };
    if let Some(body) = find_first_element(&doc.root.children, "body") {
        let mut body_style = style_for(body, ctx.styles, &inherited, &ctx.ancestor_stack);
        if body_style.bg.is_none() {
            body_style.bg = Some(egui::Color32::WHITE);
        }
        if !matches!(body_style.display, Some(Display::None)) {
            render_box(ui, &body_style, |ui| {
                ctx.ancestor_stack.push(selector_subject(body));
                for node in &body.children {
                    render_node(ui, node, &mut ctx, &body_style);
                }
                ctx.ancestor_stack.pop();
            });
        }
    } else {
        let root_style = StyleProps {
            bg: Some(egui::Color32::WHITE),
            color: Some(egui::Color32::BLACK),
            ..StyleProps::default()
        };
        render_box(ui, &root_style, |ui| {
            for node in &doc.root.children {
                render_node(ui, node, &mut ctx, &inherited);
            }
        });
    }
}

fn render_node(ui: &mut egui::Ui, node: &HtmlNode, ctx: &mut Ctx<'_>, inherited: &StyleProps) {
    match node {
        HtmlNode::Text(t) => render_text(ui, t, inherited, TextEffects::default()),
        HtmlNode::Element(el) => render_element(ui, el, ctx, inherited),
    }
}

fn render_element(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, inherited: &StyleProps) {
    let tag = canonical_element_tag(el.tag.as_str());
    if is_non_rendered_element_tag(tag) {
        return;
    }

    if element_has_hidden_semantics(el) {
        return;
    }

    let mut style = style_for(el, ctx.styles, inherited, &ctx.ancestor_stack);
    apply_semantic_text_style(tag, &mut style);
    if style_suppresses_rendering(&style) || is_likely_screen_reader_only(&style) {
        return;
    }

    ctx.ancestor_stack.push(selector_subject(el));
    match tag {
        "h1" => render_heading(ui, el, &style, 32.0),
        "h2" => render_heading(ui, el, &style, 28.0),
        "h3" => render_heading(ui, el, &style, 24.0),
        "h4" => render_heading(ui, el, &style, 20.0),
        "h5" => render_heading(ui, el, &style, 18.0),
        "h6" => render_heading(ui, el, &style, 16.0),
        "hr" => render_horizontal_rule(ui, &style),
        "p" => {
            render_box(ui, &style, |ui| {
                if element_has_only_text_children(&el.children) {
                    let text = collect_text(&el.children);
                    if !text.trim().is_empty() {
                        render_text_block(ui, &text, &style, TextEffects::default());
                    }
                } else {
                    render_inline_wrapped(ui, &el.children, ctx, &style);
                }
            });
            add_default_bottom_spacing(ui, &style, 4.0);
        }
        "br" => ui.add_space(2.0),
        "pre" => render_pre(ui, el, &style),
        "blockquote" => render_blockquote(ui, el, ctx, &style),
        "details" => render_details(ui, el, ctx, &style),
        "summary" => render_summary(ui, el, ctx, &style),
        "dl" => render_definition_list(ui, el, ctx, &style),
        "dt" => render_definition_term(ui, el, ctx, &style),
        "dd" => render_definition_description(ui, el, ctx, &style),
        "form" => render_form(ui, el, ctx, &style),
        "center" => render_center(ui, el, ctx, &style),
        "dialog" => {
            render_box(ui, &style, |ui| {
                for child in &el.children {
                    render_node(ui, child, ctx, &style);
                }
            });
            add_default_bottom_spacing(ui, &style, 2.0);
        }
        "table" => render_table(ui, el, ctx, &style),
        "tr" => render_table_row(ui, el, ctx, &style, 0.0, 0.0),
        "td" | "th" => render_table_cell(ui, el, ctx, &style, None, ui.available_width(), 0.0),
        "ul" => render_list(ui, el, false, ctx, &style),
        "ol" => render_list(ui, el, true, ctx, &style),
        "menu" | "dir" => render_list(ui, el, false, ctx, &style),
        "li" => {
            render_box(ui, &style, |ui| {
                if is_rtl_layout(&style) {
                    ui.horizontal_wrapped(|ui| {
                        render_inline(ui, &el.children, ctx, &style);
                        ui.label("*");
                    });
                } else {
                    ui.horizontal_wrapped(|ui| {
                        ui.label("*");
                        render_inline(ui, &el.children, ctx, &style);
                    });
                }
            });
        }
        "img" => render_img(ui, el, ctx, &style),
        "input" => render_input(ui, el, ctx, &style, false),
        "keygen" => render_input(ui, el, ctx, &style, false),
        "button" => render_button(ui, el, ctx, &style, false),
        "menuitem" => render_button(ui, el, ctx, &style, false),
        "textarea" => render_textarea(ui, el, ctx, &style, false),
        "select" => render_select(ui, el, ctx, &style, false),
        "video" | "audio" | "canvas" | "svg" | "math" | "iframe" | "fencedframe" | "portal"
        | "frame" | "embed" | "object" | "applet" => render_embedded_content(ui, el, ctx, &style),
        "plaintext" | "xmp" => render_pre(ui, el, &style),
        "a" => {
            if anchor_has_element_children(el) {
                render_anchor_container(ui, el, ctx, &style);
            } else {
                render_box(ui, &style, |ui| {
                    if is_rtl_layout(&style) {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            render_link(ui, el, ctx, &style);
                        });
                    } else {
                        ui.horizontal_wrapped(|ui| render_link(ui, el, ctx, &style));
                    }
                });
            }
            add_default_bottom_spacing(ui, &style, 2.0);
        }
        _ => {
            let display = style.display.unwrap_or_else(|| {
                if is_block(tag) {
                    Display::Block
                } else {
                    Display::Inline
                }
            });
            match display {
                Display::Block => {
                    render_box(ui, &style, |ui| {
                        for child in &el.children {
                            render_node(ui, child, ctx, &style);
                        }
                    });
                    add_default_bottom_spacing(ui, &style, 2.0);
                }
                Display::Flex => {
                    render_flex(ui, el, ctx, &style);
                    add_default_bottom_spacing(ui, &style, 2.0);
                }
                Display::Grid => {
                    render_grid(ui, el, ctx, &style);
                    add_default_bottom_spacing(ui, &style, 2.0);
                }
                Display::Inline => {
                    ui.horizontal_wrapped(|ui| render_inline(ui, &el.children, ctx, &style));
                }
                Display::None => {}
            }
        }
    }
    ctx.ancestor_stack.pop();
}

fn apply_semantic_text_style(tag: &str, style: &mut StyleProps) {
    match tag {
        "strong" | "b" => {
            if style.bold.is_none() {
                style.bold = Some(true);
            }
        }
        "em" | "i" | "cite" => {
            if style.italic.is_none() {
                style.italic = Some(true);
            }
        }
        "u" | "ins" => {
            if style.underline.is_none() {
                style.underline = Some(true);
            }
        }
        "s" | "strike" | "del" => {
            if style.strike.is_none() {
                style.strike = Some(true);
            }
        }
        _ => {}
    }
}

fn render_inline(ui: &mut egui::Ui, nodes: &[HtmlNode], ctx: &mut Ctx<'_>, inherited: &StyleProps) {
    for node in nodes {
        match node {
            HtmlNode::Text(t) => render_text(ui, t, inherited, TextEffects::default()),
            HtmlNode::Element(el) => {
                let tag = canonical_element_tag(el.tag.as_str());
                let style = style_for(el, ctx.styles, inherited, &ctx.ancestor_stack);
                if is_non_rendered_element_tag(tag)
                    || element_has_hidden_semantics(el)
                    || style_suppresses_rendering(&style)
                    || is_likely_screen_reader_only(&style)
                {
                    continue;
                }
                match tag {
                    "strong" | "b" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    strong: true,
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "em" | "i" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    italics: true,
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "u" | "ins" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    underline: true,
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "del" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    strike: true,
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "s" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    strike: true,
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "mark" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    mark: true,
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "small" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    small: true,
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "sub" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    small: true,
                                    script: Some(ScriptPosition::Sub),
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "sup" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    small: true,
                                    script: Some(ScriptPosition::Sup),
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "code" | "tt" | "kbd" | "samp" | "var" => {
                        let t = collect_text(&el.children);
                        if !t.is_empty() {
                            render_text(
                                ui,
                                &t,
                                &style,
                                TextEffects {
                                    mono: true,
                                    ..TextEffects::default()
                                },
                            );
                        }
                    }
                    "q" => {
                        let t = collapse_whitespace(&collect_text(&el.children));
                        if !t.is_empty() {
                            let quoted = format!("\"{t}\"");
                            render_text(ui, &quoted, &style, TextEffects::default());
                        }
                    }
                    "br" => {
                        ui.label("");
                    }
                    "a" => {
                        if anchor_has_element_children(el) {
                            render_element(ui, el, ctx, inherited);
                        } else {
                            render_link(ui, el, ctx, &style);
                        }
                    }
                    "img" => render_img(ui, el, ctx, &style),
                    "input" => render_input(ui, el, ctx, &style, true),
                    "keygen" => render_input(ui, el, ctx, &style, true),
                    "button" => render_button(ui, el, ctx, &style, true),
                    "menuitem" => render_button(ui, el, ctx, &style, true),
                    "textarea" => render_textarea(ui, el, ctx, &style, true),
                    "select" => render_select(ui, el, ctx, &style, true),
                    _ => {
                        let display = style.display.unwrap_or_else(|| {
                            if is_block(tag) {
                                Display::Block
                            } else {
                                Display::Inline
                            }
                        });
                        if matches!(display, Display::Block | Display::Flex | Display::Grid) {
                            render_element(ui, el, ctx, inherited);
                        } else {
                            ctx.ancestor_stack.push(selector_subject(el));
                            render_inline(ui, &el.children, ctx, &style);
                            ctx.ancestor_stack.pop();
                        }
                    }
                }
            }
        }
    }
}
fn render_link(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let href = attr(el, "href").map(ToOwned::to_owned);
    let disabled = has_attr(el, "disabled") || has_attr(el, "inert");
    let text = {
        let raw = collapse_whitespace(&collect_text(&el.children));
        if raw.is_empty() {
            attr(el, "aria-label")
                .map(collapse_whitespace)
                .filter(|label| !label.is_empty())
                .or_else(|| {
                    attr(el, "title")
                        .map(collapse_whitespace)
                        .filter(|label| !label.is_empty())
                })
        } else {
            Some(raw)
        }
    };

    let Some(text) = text else {
        return;
    };

    if disabled {
        render_text(ui, &text, style, TextEffects::default());
        return;
    }

    if let Some(href) = href {
        if let Some(url) = resolve_link(ctx.base_url, &href) {
            let rich = build_rich_text(
                text,
                style,
                TextEffects {
                    underline: true,
                    ..TextEffects::default()
                },
            );
            if ui.link(rich).clicked() {
                emit_inline_event(ctx, DomEventKind::Click, el, "onclick");
                ctx.action.navigate_to = Some(url);
            }
            return;
        } else {
            render_text(ui, &text, style, TextEffects::default());
            return;
        }
    }

    render_text(ui, &text, style, TextEffects::default());
}

fn anchor_has_element_children(el: &HtmlElement) -> bool {
    el.children
        .iter()
        .any(|node| matches!(node, HtmlNode::Element(_)))
}

fn render_anchor_container(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
) {
    let href = attr(el, "href").and_then(|value| resolve_link(ctx.base_url, value));
    let disabled = has_attr(el, "disabled") || has_attr(el, "inert");
    let mut container_style = style.clone();
    if matches!(container_style.display, None | Some(Display::Inline)) {
        // Grid/flex items are blockified by CSS layout.
        container_style.display = Some(Display::Block);
    }

    render_box(ui, &container_style, |ui| {
        let id = ui.next_auto_id();
        let inner = ui.scope(|ui| {
            for child in &el.children {
                render_node(ui, child, ctx, &container_style);
            }
        });

        if !disabled && let Some(url) = href.as_ref() {
            let response = ui.interact(inner.response.rect, id, egui::Sense::click());
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                emit_inline_event(ctx, DomEventKind::Click, el, "onclick");
                ctx.action.navigate_to = Some(url.clone());
            }
        }
    });
}

fn render_heading(ui: &mut egui::Ui, el: &HtmlElement, style: &StyleProps, default_size: f32) {
    let text = collapse_whitespace(&collect_text(&el.children));
    if text.is_empty() {
        return;
    }

    let mut rich = egui::RichText::new(text)
        .strong()
        .size(style.font_size.unwrap_or(default_size));
    if let Some(v) = style.color {
        rich = rich.color(v);
    }

    render_box(ui, style, |ui| {
        add_aligned_label(ui, egui::Label::new(rich), style);
    });
    add_default_bottom_spacing(ui, style, 4.0);
}

fn render_pre(ui: &mut egui::Ui, el: &HtmlElement, style: &StyleProps) {
    let text = collect_text(&el.children);
    if text.is_empty() {
        return;
    }

    let mut frame = egui::Frame::NONE
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 58, 72)))
        .inner_margin(style.padding.max_or(8.0).max(0.0));

    frame = frame.fill(style.bg.unwrap_or(egui::Color32::from_rgb(16, 20, 26)));

    frame.show(ui, |ui| {
        let mut rich = egui::RichText::new(text).monospace();
        if let Some(v) = style.color {
            rich = rich.color(v);
        }
        if let Some(v) = style.font_size {
            rich = rich.size(v);
        }
        ui.label(rich);
    });

    if let Some(margin_bottom) = style.margin.bottom {
        if margin_bottom > 0.0 {
            ui.add_space(margin_bottom);
        }
    } else {
        ui.add_space(4.0);
    }
}

fn render_horizontal_rule(ui: &mut egui::Ui, style: &StyleProps) {
    render_box(ui, style, |ui| {
        let width = style.width.unwrap_or(ui.available_width()).max(1.0);
        let thickness = style.border_width.top.unwrap_or(1.0).clamp(1.0, 6.0);
        let color = style
            .border_color
            .or(style.color)
            .unwrap_or(egui::Color32::from_gray(96));
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(width, (thickness + 4.0).max(4.0)),
            egui::Sense::hover(),
        );
        let y = rect.center().y;
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(thickness, color),
        );
    });
    add_default_bottom_spacing(ui, style, 6.0);
}

fn render_blockquote(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let border = style
        .border_color
        .unwrap_or(egui::Color32::from_rgb(110, 130, 154));
    let fill = style
        .bg
        .unwrap_or(egui::Color32::from_rgba_unmultiplied(170, 170, 170, 22));
    render_box(ui, style, |ui| {
        egui::Frame::NONE
            .fill(fill)
            .stroke(egui::Stroke::new(1.0, border))
            .inner_margin(style.padding.max_or(8.0).max(6.0))
            .show(ui, |ui| {
                for child in &el.children {
                    render_node(ui, child, ctx, style);
                }
            });
    });
    add_default_bottom_spacing(ui, style, 4.0);
}

fn render_details(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let summary = el
        .children
        .iter()
        .find_map(|child| {
            let HtmlNode::Element(summary) = child else {
                return None;
            };
            if summary.tag != "summary" {
                return None;
            }
            let text = collapse_whitespace(&collect_text(&summary.children));
            if text.is_empty() {
                Some("Details".to_owned())
            } else {
                Some(text)
            }
        })
        .unwrap_or_else(|| "Details".to_owned());
    let default_open = attr(el, "open").is_some();

    render_box(ui, style, |ui| {
        egui::CollapsingHeader::new(summary)
            .default_open(default_open)
            .show(ui, |ui| {
                for child in &el.children {
                    let HtmlNode::Element(summary) = child else {
                        render_node(ui, child, ctx, style);
                        continue;
                    };
                    if summary.tag == "summary" {
                        continue;
                    }
                    render_node(ui, child, ctx, style);
                }
            });
    });
    add_default_bottom_spacing(ui, style, 2.0);
}

fn render_summary(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let text = collapse_whitespace(&collect_text(&el.children));
    if !text.is_empty() {
        render_box(ui, style, |ui| {
            ui.label(build_rich_text(
                text,
                style,
                TextEffects {
                    strong: true,
                    ..TextEffects::default()
                },
            ));
        });
    } else {
        render_box(ui, style, |ui| {
            for child in &el.children {
                render_node(ui, child, ctx, style);
            }
        });
    }
    add_default_bottom_spacing(ui, style, 2.0);
}

fn render_definition_list(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
) {
    render_box(ui, style, |ui| {
        for child in &el.children {
            let HtmlNode::Element(def) = child else {
                render_node(ui, child, ctx, style);
                continue;
            };
            match def.tag.as_str() {
                "dt" => render_definition_term(ui, def, ctx, style),
                "dd" => render_definition_description(ui, def, ctx, style),
                _ => render_node(ui, child, ctx, style),
            }
        }
    });
    add_default_bottom_spacing(ui, style, 2.0);
}

fn render_definition_term(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
) {
    let mut term_style = style.clone();
    if term_style.bold.is_none() {
        term_style.bold = Some(true);
    }
    render_box(ui, &term_style, |ui| {
        render_inline_wrapped(ui, &el.children, ctx, &term_style);
    });
    add_default_bottom_spacing(ui, &term_style, 1.0);
}

fn render_definition_description(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
) {
    let indent = style.padding.left.unwrap_or(18.0).max(12.0);
    render_box(ui, style, |ui| {
        ui.horizontal_wrapped(|ui| {
            if is_rtl_layout(style) {
                render_inline_wrapped(ui, &el.children, ctx, style);
                ui.add_space(indent);
            } else {
                ui.add_space(indent);
                render_inline_wrapped(ui, &el.children, ctx, style);
            }
        });
    });
    add_default_bottom_spacing(ui, style, 2.0);
}

fn render_embedded_content(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
) {
    let source = attr(el, "src")
        .or_else(|| attr(el, "data"))
        .or_else(|| attr(el, "poster"))
        .map(ToOwned::to_owned);
    let title = attr(el, "title")
        .or_else(|| attr(el, "aria-label"))
        .unwrap_or("");
    let label = format!("<{}> content placeholder", el.tag);

    render_box(ui, style, |ui| {
        egui::Frame::NONE
            .fill(
                style
                    .bg
                    .unwrap_or(egui::Color32::from_rgba_unmultiplied(140, 140, 140, 22)),
            )
            .stroke(egui::Stroke::new(
                1.0,
                style
                    .border_color
                    .unwrap_or(egui::Color32::from_rgb(120, 120, 120)),
            ))
            .inner_margin(style.padding.max_or(8.0).max(6.0))
            .show(ui, |ui| {
                ui.label(build_rich_text(
                    label,
                    style,
                    TextEffects {
                        strong: true,
                        ..TextEffects::default()
                    },
                ));
                if !title.trim().is_empty() {
                    ui.label(format!("Title: {}", title.trim()));
                }
                if let Some(source) = source.as_deref() {
                    if let Some(url) = resolve_link(ctx.base_url, source) {
                        if ui.link(format!("Open source: {url}")).clicked() {
                            ctx.action.navigate_to = Some(url);
                        }
                    } else {
                        ui.label(format!("Source: {source}"));
                    }
                }
            });
    });
    add_default_bottom_spacing(ui, style, 2.0);
}

fn render_form(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let key = form_runtime_key(el);
    let action_url = attr(el, "action")
        .and_then(|value| resolve_link(ctx.base_url, value))
        .unwrap_or_else(|| ctx.base_url.to_owned());
    let method = attr(el, "method")
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "get".to_owned());
    let onsubmit = attr(el, "onsubmit").map(ToOwned::to_owned);

    ctx.form_stack.push(FormRuntime {
        key: key.clone(),
        action_url,
        method,
        form_id: attr(el, "id").map(ToOwned::to_owned),
        onsubmit,
    });
    ctx.form_fields.entry(key).or_default();

    render_box(ui, style, |ui| {
        for child in &el.children {
            render_node(ui, child, ctx, style);
        }
    });
    ctx.form_stack.pop();
    add_default_bottom_spacing(ui, style, 2.0);
}

fn render_center(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let mut centered = style.clone();
    if centered.text_align.is_none() {
        centered.text_align = Some(TextAlign::Center);
    }

    render_box(ui, &centered, |ui| {
        ui.vertical_centered(|ui| {
            for child in &el.children {
                render_node(ui, child, ctx, &centered);
            }
        });
    });
    add_default_bottom_spacing(ui, &centered, 2.0);
}

fn render_table(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let mut rows = Vec::new();
    collect_table_rows(&el.children, &mut rows);
    let cell_spacing = attr(el, "cellspacing")
        .and_then(parse_html_length)
        .unwrap_or(0.0)
        .max(0.0);
    let cell_padding = attr(el, "cellpadding")
        .and_then(parse_html_length)
        .unwrap_or(0.0)
        .max(0.0);
    let mut table_style = apply_html_alignment_attr(el, style);
    if table_style.display.is_none() {
        table_style.display = Some(Display::Block);
    }

    render_box(ui, &table_style, |ui| {
        for child in &el.children {
            let HtmlNode::Element(caption) = child else {
                continue;
            };
            if canonical_element_tag(caption.tag.as_str()) == "caption" {
                render_node(ui, child, ctx, &table_style);
            }
        }

        for (index, row) in rows.iter().enumerate() {
            if index > 0 && cell_spacing > 0.0 {
                ui.add_space(cell_spacing);
            }
            render_table_row(ui, row, ctx, &table_style, cell_spacing, cell_padding);
        }
    });
    add_default_bottom_spacing(ui, &table_style, 2.0);
}

fn collect_table_rows<'a>(nodes: &'a [HtmlNode], out: &mut Vec<&'a HtmlElement>) {
    for node in nodes {
        let HtmlNode::Element(el) = node else {
            continue;
        };

        match el.tag.as_str() {
            "tr" => out.push(el),
            "thead" | "tbody" | "tfoot" => collect_table_rows(&el.children, out),
            _ => {}
        }
    }
}

fn render_table_row(
    ui: &mut egui::Ui,
    row: &HtmlElement,
    ctx: &mut Ctx<'_>,
    inherited: &StyleProps,
    cell_spacing: f32,
    cell_padding: f32,
) {
    let mut row_style = style_for(row, ctx.styles, inherited, &ctx.ancestor_stack);
    row_style = apply_html_alignment_attr(row, &row_style);

    let mut cells = Vec::new();
    for child in &row.children {
        let HtmlNode::Element(cell) = child else {
            continue;
        };
        if matches!(cell.tag.as_str(), "td" | "th") {
            cells.push(cell);
        }
    }

    ctx.ancestor_stack.push(selector_subject(row));
    render_box(ui, &row_style, |ui| {
        ui.horizontal(|ui| {
            let row_width = ui.available_width().max(1.0);
            let spacing_total = if cells.len() > 1 {
                cell_spacing * (cells.len().saturating_sub(1) as f32)
            } else {
                0.0
            };

            let mut resolved_widths = Vec::with_capacity(cells.len());
            let mut cell_spans = Vec::with_capacity(cells.len());
            let mut auto_width_indices = Vec::new();
            let mut fixed_total = 0.0_f32;

            for (cell_index, cell) in cells.iter().enumerate() {
                let mut cell_style = style_for(cell, ctx.styles, &row_style, &ctx.ancestor_stack);
                cell_style = apply_html_alignment_attr(cell, &cell_style);
                let colspan = parse_usize_attr(cell, "colspan").unwrap_or(1).max(1);

                let width_from_css = cell_style.width.or_else(|| {
                    cell_style
                        .width_percent
                        .map(|percent| row_width * (percent / 100.0))
                });
                let width_from_attr = attr(cell, "width")
                    .and_then(|raw| parse_html_dimension(raw, row_width))
                    .map(|value| value.max(1.0));

                let mut resolved = width_from_css.or(width_from_attr);
                if let Some(width) = resolved {
                    let mut clamped = width.max(1.0);
                    if let Some(min_width) = cell_style.min_width {
                        clamped = clamped.max(min_width.max(0.0));
                    }
                    if let Some(max_width) = cell_style.max_width {
                        clamped = clamped.min(max_width.max(1.0));
                    }
                    fixed_total += clamped;
                    resolved = Some(clamped);
                } else {
                    auto_width_indices.push(cell_index);
                }

                resolved_widths.push(resolved);
                cell_spans.push(colspan);
            }

            if !auto_width_indices.is_empty() {
                let remaining = (row_width - spacing_total - fixed_total).max(1.0);
                let auto_span_total = auto_width_indices
                    .iter()
                    .map(|index| cell_spans[*index])
                    .sum::<usize>()
                    .max(1);
                let per_auto = (remaining / auto_span_total as f32).max(1.0);
                for index in auto_width_indices {
                    let span = cell_spans.get(index).copied().unwrap_or(1).max(1);
                    resolved_widths[index] = Some((per_auto * span as f32).max(1.0));
                }
            }

            if is_rtl_layout(&row_style) {
                for (index, cell) in cells.iter().rev().enumerate() {
                    if index > 0 && cell_spacing > 0.0 {
                        ui.add_space(cell_spacing);
                    }
                    let width_index = cells.len().saturating_sub(index + 1);
                    render_table_cell(
                        ui,
                        cell,
                        ctx,
                        &row_style,
                        resolved_widths.get(width_index).copied().flatten(),
                        row_width,
                        cell_padding,
                    );
                }
            } else {
                for (index, cell) in cells.iter().enumerate() {
                    if index > 0 && cell_spacing > 0.0 {
                        ui.add_space(cell_spacing);
                    }
                    render_table_cell(
                        ui,
                        cell,
                        ctx,
                        &row_style,
                        resolved_widths.get(index).copied().flatten(),
                        row_width,
                        cell_padding,
                    );
                }
            }
        });
    });
    ctx.ancestor_stack.pop();
}

fn render_table_cell(
    ui: &mut egui::Ui,
    cell: &HtmlElement,
    ctx: &mut Ctx<'_>,
    inherited: &StyleProps,
    resolved_width: Option<f32>,
    row_available_width: f32,
    cell_padding: f32,
) {
    let mut cell_style = style_for(cell, ctx.styles, inherited, &ctx.ancestor_stack);
    cell_style = apply_html_alignment_attr(cell, &cell_style);
    if cell_padding > 0.0 && cell_style.padding.max_or(0.0) <= 0.0 {
        cell_style.padding = Edges::all(cell_padding);
    }

    if cell_style.width.is_none() && resolved_width.is_none() {
        if let Some(width) =
            attr(cell, "width").and_then(|raw| parse_html_dimension(raw, row_available_width))
        {
            cell_style.width = Some(width.max(1.0));
        }
    } else if cell_style.width.is_none() {
        cell_style.width = resolved_width;
    }

    let nowrap = attr(cell, "nowrap").is_some();

    ctx.ancestor_stack.push(selector_subject(cell));
    render_box(ui, &cell_style, |ui| {
        if nowrap {
            render_inline(ui, &cell.children, ctx, &cell_style);
        } else {
            for child in &cell.children {
                render_node(ui, child, ctx, &cell_style);
            }
        }
    });
    ctx.ancestor_stack.pop();
}

fn render_list(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    numbered: bool,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
) {
    let reversed = numbered && has_attr(el, "reversed");
    let mut index = if numbered {
        parse_usize_attr(el, "start").unwrap_or(1)
    } else {
        1
    };
    let marker_kind = style
        .list_style_type
        .clone()
        .or_else(|| {
            attr(el, "type")
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        })
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());

    let mut items: Vec<(&HtmlElement, StyleProps)> = Vec::new();
    for child in &el.children {
        let HtmlNode::Element(item) = child else {
            continue;
        };
        if item.tag != "li" {
            continue;
        }
        let item_style = style_for(item, ctx.styles, style, &ctx.ancestor_stack);
        if matches!(item_style.display, Some(Display::None)) {
            continue;
        }
        items.push((item, item_style));
    }

    if numbered && reversed && !has_attr(el, "start") {
        let list_len = items.len();
        index = list_len.max(1);
    }

    let is_unordered_none = !numbered
        && marker_kind
            .as_deref()
            .is_some_and(|kind| kind.eq_ignore_ascii_case("none"));
    let has_inline_items = items
        .iter()
        .any(|(_, item_style)| matches!(item_style.display, Some(Display::Inline)));
    if is_unordered_none && has_inline_items {
        render_box(ui, style, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (item, item_style) in &items {
                    ctx.ancestor_stack.push(selector_subject(item));
                    render_box(ui, item_style, |ui| {
                        render_inline(ui, &item.children, ctx, item_style);
                    });
                    ctx.ancestor_stack.pop();
                }
            });
        });
        add_default_bottom_spacing(ui, style, 4.0);
        return;
    }

    render_box(ui, style, |ui| {
        for (item, item_style) in &items {
            ui.horizontal_wrapped(|ui| {
                let mark = if numbered {
                    ordered_list_marker(index, marker_kind.as_deref())
                } else {
                    unordered_list_marker(marker_kind.as_deref())
                };
                ctx.ancestor_stack.push(selector_subject(item));
                if is_rtl_layout(&item_style) {
                    render_inline(ui, &item.children, ctx, &item_style);
                    if !mark.is_empty() {
                        ui.label(mark);
                    }
                } else {
                    if !mark.is_empty() {
                        ui.label(mark);
                    }
                    render_inline(ui, &item.children, ctx, &item_style);
                }
                ctx.ancestor_stack.pop();
            });
            if numbered && reversed {
                index = index.saturating_sub(1);
            } else {
                index = index.saturating_add(1);
            }
        }
    });

    add_default_bottom_spacing(ui, style, 4.0);
}

fn ordered_list_marker(index: usize, marker_type: Option<&str>) -> String {
    let marker = marker_type
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("1");

    match marker {
        "a" | "lower-alpha" => format!("{}.", alpha_list_marker(index, false)),
        "A" | "upper-alpha" => format!("{}.", alpha_list_marker(index, true)),
        "i" | "lower-roman" => format!("{}.", roman_list_marker(index, false)),
        "I" | "upper-roman" => format!("{}.", roman_list_marker(index, true)),
        "none" => String::new(),
        _ => format!("{}.", index),
    }
}

fn unordered_list_marker(marker_type: Option<&str>) -> String {
    match marker_type
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("circle") => "\u{25E6}".to_owned(),
        Some("square") => "\u{25AA}".to_owned(),
        Some("none") => String::new(),
        _ => "\u{2022}".to_owned(),
    }
}

fn alpha_list_marker(index: usize, uppercase: bool) -> String {
    if index == 0 {
        return "0".to_owned();
    }

    let mut value = index;
    let mut out = String::new();
    while value > 0 {
        let rem = (value - 1) % 26;
        let rem_u8 = u8::try_from(rem).unwrap_or(0);
        let base = if uppercase { b'A' } else { b'a' };
        out.insert(0, char::from(base + rem_u8));
        value = (value - 1) / 26;
    }
    out
}

fn roman_list_marker(index: usize, uppercase: bool) -> String {
    if index == 0 {
        return "0".to_owned();
    }

    let mut value = index.min(3999);
    let mut out = String::new();
    let parts: &[(usize, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    for (weight, token) in parts {
        while value >= *weight {
            out.push_str(token);
            value -= *weight;
        }
    }

    if uppercase {
        out
    } else {
        out.to_ascii_lowercase()
    }
}

fn render_img(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let src = image_source_attr(el).map(ToOwned::to_owned);
    let alt = attr(el, "alt").unwrap_or("image").to_owned();
    let resolved = src
        .as_deref()
        .and_then(|value| resolve_link(ctx.base_url, value));
    let fallback_width = ui.available_width().clamp(120.0, 420.0);
    let available_width = ui.available_width().max(1.0);

    let mut intrinsic_width: Option<f32> = None;
    let mut intrinsic_height: Option<f32> = None;
    if let Some(url) = resolved.as_deref() {
        if let Some(render_image) = ctx.resources.images.get(url) {
            intrinsic_width = Some(render_image.size.x.max(1.0));
            intrinsic_height = Some(render_image.size.y.max(1.0));
        }
    }

    let mut width = style
        .width
        .or_else(|| {
            style
                .width_percent
                .map(|percent| available_width * (percent / 100.0))
        })
        .or_else(|| attr(el, "width").and_then(parse_length))
        .or(intrinsic_width)
        .unwrap_or(fallback_width)
        .max(40.0);

    let mut height = style
        .height
        .or_else(|| attr(el, "height").and_then(parse_length))
        .or(intrinsic_height)
        .unwrap_or((width * 0.55).max(60.0))
        .max(30.0);
    if let Some(min_height) = style.min_height {
        height = height.max(min_height.max(0.0));
    }
    if let Some(max_height) = style.max_height {
        height = height.min(max_height.max(30.0));
    }

    if style.width.is_some() && style.height.is_none() {
        if let (Some(iw), Some(ih)) = (intrinsic_width, intrinsic_height) {
            if iw > 0.0 {
                height = (width * (ih / iw)).max(1.0);
            }
        }
    } else if style.width.is_none() && style.height.is_some() {
        if let (Some(iw), Some(ih)) = (intrinsic_width, intrinsic_height) {
            if ih > 0.0 {
                width = (height * (iw / ih)).max(1.0);
            }
        }
    }

    if let Some(url) = resolved.as_deref() {
        if let Some(render_image) = ctx.resources.images.get(url) {
            let mut image =
                egui::Image::new((render_image.texture_id, egui::vec2(width, height)));
            if let Some(radius) = style.border_radius {
                image = image.corner_radius(radius.clamp(0.0, 255.0).round() as u8);
            }
            if effective_opacity(style) < 0.999 {
                image = image.tint(color_with_effective_opacity(egui::Color32::WHITE, style));
            }
            ui.add(image);
            let margin_bottom = style.margin.bottom_or(0.0).max(0.0);
            if margin_bottom > 0.0 {
                ui.add_space(margin_bottom);
            } else {
                ui.add_space(2.0);
            }
            return;
        }
    }

    let fill = color_with_effective_opacity(
        style.bg.unwrap_or(egui::Color32::from_rgb(21, 26, 34)),
        style,
    );

    egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::new(
            1.0,
            color_with_effective_opacity(egui::Color32::from_rgb(66, 78, 95), style),
        ))
        .corner_radius(style.border_radius.unwrap_or(0.0).clamp(0.0, 255.0).round() as u8)
        .inner_margin(style.padding.max_or(10.0).max(0.0))
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(width, height));
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Image").strong());
                ui.label(egui::RichText::new(alt.clone()).italics());
            });
        });

    if let Some(url) = resolved {
        ui.horizontal_wrapped(|ui| {
            ui.label("src:");
            if ui.link(&url).clicked() {
                ctx.action.navigate_to = Some(url.clone());
            }
        });
    }

    if let Some(margin_bottom) = style.margin.bottom {
        if margin_bottom > 0.0 {
            ui.add_space(margin_bottom);
        }
    } else {
        ui.add_space(2.0);
    }
}

fn render_text(ui: &mut egui::Ui, text: &str, style: &StyleProps, effects: TextEffects) {
    let wrap_mode = match effective_white_space_mode(style, &effects) {
        WhiteSpaceMode::NoWrap | WhiteSpaceMode::Pre => egui::TextWrapMode::Extend,
        WhiteSpaceMode::Normal | WhiteSpaceMode::PreWrap | WhiteSpaceMode::PreLine => {
            egui::TextWrapMode::Wrap
        }
    };
    let label = egui::Label::new(build_rich_text(text.to_owned(), style, effects)).wrap_mode(wrap_mode);
    ui.add(label);
}

fn render_text_block(ui: &mut egui::Ui, text: &str, style: &StyleProps, effects: TextEffects) {
    let wrap_mode = match effective_white_space_mode(style, &effects) {
        WhiteSpaceMode::NoWrap | WhiteSpaceMode::Pre => egui::TextWrapMode::Extend,
        WhiteSpaceMode::Normal | WhiteSpaceMode::PreWrap | WhiteSpaceMode::PreLine => {
            egui::TextWrapMode::Wrap
        }
    };
    let label = egui::Label::new(build_rich_text(text.to_owned(), style, effects)).wrap_mode(wrap_mode);
    add_aligned_label(ui, label, style);
}

fn add_aligned_label(ui: &mut egui::Ui, label: egui::Label, style: &StyleProps) {
    let horizontal_align = match style.text_align.unwrap_or(TextAlign::Left) {
        TextAlign::Left | TextAlign::Justify => egui::Align::Min,
        TextAlign::Center => egui::Align::Center,
        TextAlign::Right => egui::Align::Max,
    };
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width().max(1.0), 0.0),
        egui::Layout::top_down(horizontal_align),
        |ui| {
            ui.add(label);
        },
    );
}

fn element_has_only_text_children(children: &[HtmlNode]) -> bool {
    children.iter().all(|node| match node {
        HtmlNode::Text(_) => true,
        HtmlNode::Element(el) => canonical_element_tag(el.tag.as_str()) == "br",
    })
}

fn render_inline_wrapped(
    ui: &mut egui::Ui,
    nodes: &[HtmlNode],
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
) {
    if is_rtl_layout(style) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.horizontal_wrapped(|ui| render_inline(ui, nodes, ctx, style));
        });
    } else {
        ui.horizontal_wrapped(|ui| render_inline(ui, nodes, ctx, style));
    }
}

fn is_rtl_layout(style: &StyleProps) -> bool {
    matches!(style.text_align, Some(TextAlign::Right))
}

fn render_input(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
    inline_mode: bool,
) {
    let input_type = attr(el, "type").unwrap_or("text").to_ascii_lowercase();
    let disabled = has_attr(el, "disabled") || has_attr(el, "inert");
    let read_only = has_attr(el, "readonly");
    let max_length = parse_usize_attr(el, "maxlength");
    if input_type == "hidden" {
        if !disabled && let (Some(_), Some(name)) = (ctx.form_stack.last(), attr(el, "name")) {
            let value = attr(el, "value").unwrap_or("").to_owned();
            set_active_form_field(ctx, name, Some(value));
        }
        return;
    }

    let value = attr(el, "value").unwrap_or("").to_owned();
    let placeholder = attr(el, "placeholder").unwrap_or("").to_owned();
    let size_attr_width = attr(el, "size")
        .and_then(|raw| raw.parse::<f32>().ok())
        .map(|size| (size * 9.0 + 24.0).max(60.0));
    let available_width = ui.available_width().max(1.0);
    let submit_fallback = ((value.chars().count() as f32) * 9.0 + 24.0).clamp(60.0, 240.0);
    let mut width = style
        .width
        .or_else(|| {
            style
                .width_percent
                .map(|percent| available_width * (percent / 100.0))
        })
        .or(size_attr_width)
        .unwrap_or(if matches!(input_type.as_str(), "submit" | "button") {
            submit_fallback
        } else {
            260.0
        })
        .max(60.0);
    if let Some(min_width) = style.min_width {
        width = width.max(min_width.max(0.0));
    }
    if let Some(max_width) = style.max_width {
        width = width.min(max_width.max(60.0));
    }
    let mut height = style.height.unwrap_or(30.0).max(24.0);
    if let Some(min_height) = style.min_height {
        height = height.max(min_height.max(0.0));
    }
    if let Some(max_height) = style.max_height {
        height = height.min(max_height.max(24.0));
    }

    let text_control_style = tune_text_control_style(style);
    let state_key = form_control_state_key(ctx.base_url, el, &input_type);
    let mut render_control = |ui: &mut egui::Ui| match input_type.as_str() {
        "submit" | "button" => {
            let label = if value.is_empty() {
                "Button".to_owned()
            } else {
                value.clone()
            };
            let rich = build_rich_text(label, style, TextEffects::default());
            let response = ui
                .add_enabled_ui(!disabled, |ui| {
                    with_form_control_visuals(ui, style, |ui| {
                        ui.add_sized([width, height], egui::Button::new(rich))
                    })
                })
                .inner;
            if disabled {
                return;
            }
            if response.clicked() {
                emit_inline_event(ctx, DomEventKind::Click, el, "onclick");
                if input_type == "submit" {
                    submit_active_form(
                        ctx,
                        attr(el, "name").map(ToOwned::to_owned),
                        Some(attr(el, "value").unwrap_or("Submit").to_owned()),
                        Some(el),
                    );
                }
            }
        }
        "checkbox" | "radio" => {
            let default_checked = attr(el, "checked").is_some();
            let mut checked = ctx
                .form_state
                .get(&state_key)
                .and_then(|stored| match stored.as_str() {
                    "1" => Some(true),
                    "0" => Some(false),
                    _ => None,
                })
                .unwrap_or(default_checked);
            let response = ui
                .add_enabled_ui(!disabled, |ui| {
                    if input_type == "radio" {
                        let response = ui.radio(checked, value.clone());
                        if response.clicked() {
                            checked = true;
                        }
                        response
                    } else {
                        ui.checkbox(&mut checked, value.clone())
                    }
                })
                .inner;
            if disabled {
                if let Some(name) = attr(el, "name") {
                    set_active_form_field(ctx, name, None);
                }
                return;
            }
            ctx.form_state.insert(
                state_key.clone(),
                if checked { "1" } else { "0" }.to_owned(),
            );
            if let Some(name) = attr(el, "name") {
                if checked {
                    let checkbox_value = attr(el, "value").unwrap_or("on").to_owned();
                    set_active_form_field(ctx, name, Some(checkbox_value));
                } else {
                    set_active_form_field(ctx, name, None);
                }
            }
            if response.changed() || (input_type == "radio" && response.clicked()) {
                emit_inline_event(ctx, DomEventKind::Input, el, "oninput");
            }
            if response.clicked() {
                emit_inline_event(ctx, DomEventKind::Click, el, "onclick");
            }
        }
        _ => {
            let mut text = ctx
                .form_state
                .get(&state_key)
                .cloned()
                .unwrap_or_else(|| value.clone());
            let mut editor = egui::TextEdit::singleline(&mut text).clip_text(false);
            if !placeholder.is_empty() {
                editor = editor.hint_text(placeholder.clone());
            }
            if input_type == "password" {
                editor = editor.password(true);
            }
            editor = editor.interactive(!disabled && !read_only);
            if let Some(color) = text_control_style.color {
                editor = editor.text_color(color);
            }
            let response = ui
                .add_enabled_ui(!disabled, |ui| {
                    with_form_control_visuals(ui, &text_control_style, |ui| {
                        ui.add_sized([width, height], editor)
                    })
                })
                .inner;
            truncate_text_to_max_chars(&mut text, max_length);
            if let Some(name) = attr(el, "name")
                && !disabled
            {
                set_active_form_field(ctx, name, Some(text.clone()));
            }
            if disabled {
                if let Some(name) = attr(el, "name") {
                    set_active_form_field(ctx, name, None);
                }
            } else if response.changed() {
                emit_inline_event(ctx, DomEventKind::Input, el, "oninput");
            }
            if !disabled && response.clicked() {
                emit_inline_event(ctx, DomEventKind::Click, el, "onclick");
            }
            let pressed_enter = !disabled
                && response.lost_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if pressed_enter {
                submit_active_form(ctx, None, None, Some(el));
            }
            ctx.form_state.insert(state_key.clone(), text);
        }
    };

    if inline_mode {
        render_control(ui);
    } else {
        render_box(ui, style, render_control);
        add_default_bottom_spacing(ui, style, 2.0);
    }
}

fn render_button(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
    inline_mode: bool,
) {
    let text = collapse_whitespace(&collect_text(&el.children));
    let label = if text.is_empty() {
        "Button".to_owned()
    } else {
        text
    };
    let disabled = has_attr(el, "disabled") || has_attr(el, "inert");
    let button_type = attr(el, "type")
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "submit".to_owned());
    let available_width = ui.available_width().max(1.0);
    let mut width = style
        .width
        .or_else(|| {
            style
                .width_percent
                .map(|percent| available_width * (percent / 100.0))
        })
        .unwrap_or(((label.chars().count() as f32) * 9.0 + 24.0).clamp(60.0, 240.0))
        .max(60.0);
    if let Some(min_width) = style.min_width {
        width = width.max(min_width.max(0.0));
    }
    if let Some(max_width) = style.max_width {
        width = width.min(max_width.max(60.0));
    }
    let mut height = style.height.unwrap_or(30.0).max(24.0);
    if let Some(min_height) = style.min_height {
        height = height.max(min_height.max(0.0));
    }
    if let Some(max_height) = style.max_height {
        height = height.min(max_height.max(24.0));
    }

    let mut render_control = |ui: &mut egui::Ui| {
        let rich = build_rich_text(label.clone(), style, TextEffects::default());
        let response = ui
            .add_enabled_ui(!disabled, |ui| {
                with_form_control_visuals(ui, style, |ui| {
                    ui.add_sized([width, height], egui::Button::new(rich))
                })
            })
            .inner;
        if disabled {
            return;
        }
        if response.clicked() {
            emit_inline_event(ctx, DomEventKind::Click, el, "onclick");
            if button_type != "button" {
                submit_active_form(
                    ctx,
                    attr(el, "name").map(ToOwned::to_owned),
                    Some(attr(el, "value").unwrap_or("").to_owned()),
                    Some(el),
                );
            }
        }
    };

    if inline_mode {
        render_control(ui);
    } else {
        render_box(ui, style, render_control);
        add_default_bottom_spacing(ui, style, 2.0);
    }
}

fn render_textarea(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
    inline_mode: bool,
) {
    let disabled = has_attr(el, "disabled") || has_attr(el, "inert");
    let read_only = has_attr(el, "readonly");
    let max_length = parse_usize_attr(el, "maxlength");
    let placeholder = attr(el, "placeholder").unwrap_or("").to_owned();
    let value = attr(el, "value")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| collect_text(&el.children));
    let rows = attr(el, "rows")
        .and_then(|raw| raw.parse::<f32>().ok())
        .unwrap_or(3.0)
        .max(1.0);
    let cols = attr(el, "cols")
        .and_then(|raw| raw.parse::<f32>().ok())
        .unwrap_or(28.0)
        .max(6.0);
    let available_width = ui.available_width().max(1.0);
    let mut width = style
        .width
        .or_else(|| {
            style
                .width_percent
                .map(|percent| available_width * (percent / 100.0))
        })
        .unwrap_or(cols * 9.0 + 20.0)
        .max(100.0);
    if let Some(min_width) = style.min_width {
        width = width.max(min_width.max(0.0));
    }
    if let Some(max_width) = style.max_width {
        width = width.min(max_width.max(100.0));
    }
    let mut height = style.height.unwrap_or(rows * 22.0 + 8.0).max(40.0);
    if let Some(min_height) = style.min_height {
        height = height.max(min_height.max(0.0));
    }
    if let Some(max_height) = style.max_height {
        height = height.min(max_height.max(40.0));
    }

    let text_control_style = tune_text_control_style(style);
    let state_key = form_control_state_key(ctx.base_url, el, "textarea");
    let mut render_control = |ui: &mut egui::Ui| {
        let mut text = ctx
            .form_state
            .get(&state_key)
            .cloned()
            .unwrap_or_else(|| value.clone());
        let mut editor = egui::TextEdit::multiline(&mut text).clip_text(false);
        if !placeholder.is_empty() {
            editor = editor.hint_text(placeholder.clone());
        }
        editor = editor.interactive(!disabled && !read_only);
        if let Some(color) = text_control_style.color {
            editor = editor.text_color(color);
        }
        let response = ui
            .add_enabled_ui(!disabled, |ui| {
                with_form_control_visuals(ui, &text_control_style, |ui| {
                    ui.add_sized([width, height], editor)
                })
            })
            .inner;
        truncate_text_to_max_chars(&mut text, max_length);
        if let Some(name) = attr(el, "name")
            && !disabled
        {
            set_active_form_field(ctx, name, Some(text.clone()));
        }
        if disabled {
            if let Some(name) = attr(el, "name") {
                set_active_form_field(ctx, name, None);
            }
        } else if response.changed() {
            emit_inline_event(ctx, DomEventKind::Input, el, "oninput");
        }
        if !disabled && response.clicked() {
            emit_inline_event(ctx, DomEventKind::Click, el, "onclick");
        }
        ctx.form_state.insert(state_key.clone(), text);
    };

    if inline_mode {
        render_control(ui);
    } else {
        render_box(ui, style, render_control);
        add_default_bottom_spacing(ui, style, 2.0);
    }
}

fn render_select(
    ui: &mut egui::Ui,
    el: &HtmlElement,
    ctx: &mut Ctx<'_>,
    style: &StyleProps,
    inline_mode: bool,
) {
    let disabled = has_attr(el, "disabled") || has_attr(el, "inert");
    let multiple = has_attr(el, "multiple");
    let mut selected_labels = Vec::new();
    let mut selected_values = Vec::new();
    let mut first_label = None;
    let mut first_value = None;

    for child in &el.children {
        let HtmlNode::Element(option) = child else {
            continue;
        };
        if option.tag != "option" {
            continue;
        }
        if has_attr(option, "disabled") {
            continue;
        }

        let label = attr(option, "label")
            .map(collapse_whitespace)
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| collapse_whitespace(&collect_text(&option.children)));
        if label.is_empty() {
            continue;
        }

        let value = attr(option, "value")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| label.clone());

        if first_label.is_none() {
            first_label = Some(label.clone());
            first_value = Some(value.clone());
        }

        if has_attr(option, "selected") {
            selected_labels.push(label);
            selected_values.push(value);
            if !multiple {
                break;
            }
        }
    }

    let label = if multiple {
        if selected_labels.is_empty() {
            first_label.clone().unwrap_or_else(|| "Select".to_owned())
        } else {
            selected_labels.join(", ")
        }
    } else {
        selected_labels
            .first()
            .cloned()
            .or(first_label.clone())
            .unwrap_or_else(|| "Select".to_owned())
    };
    let field_value = if multiple {
        if selected_values.is_empty() {
            first_value
        } else {
            Some(selected_values.join(","))
        }
    } else {
        selected_values.first().cloned().or(first_value)
    };

    let available_width = ui.available_width().max(1.0);
    let mut width = style
        .width
        .or_else(|| {
            style
                .width_percent
                .map(|percent| available_width * (percent / 100.0))
        })
        .unwrap_or(160.0)
        .max(80.0);
    if let Some(min_width) = style.min_width {
        width = width.max(min_width.max(0.0));
    }
    if let Some(max_width) = style.max_width {
        width = width.min(max_width.max(80.0));
    }
    let mut height = style.height.unwrap_or(30.0).max(24.0);
    if let Some(min_height) = style.min_height {
        height = height.max(min_height.max(0.0));
    }
    if let Some(max_height) = style.max_height {
        height = height.min(max_height.max(24.0));
    }
    let control_style = tune_text_control_style(style);
    let mut render_control = |ui: &mut egui::Ui| {
        let response = ui
            .add_enabled_ui(!disabled, |ui| {
                with_form_control_visuals(ui, &control_style, |ui| {
                    let text = format!("{label} \u{25BE}");
                    ui.add_sized(
                        [width, height],
                        egui::Button::new(build_rich_text(
                            text,
                            &control_style,
                            TextEffects::default(),
                        )),
                    )
                })
            })
            .inner;
        if let Some(name) = attr(el, "name")
            && !disabled
        {
            set_active_form_field(ctx, name, field_value.clone());
        }
        if disabled {
            if let Some(name) = attr(el, "name") {
                set_active_form_field(ctx, name, None);
            }
        } else if response.clicked() {
            emit_inline_event(ctx, DomEventKind::Input, el, "oninput");
            emit_inline_event(ctx, DomEventKind::Click, el, "onclick");
        }
    };

    if inline_mode {
        render_control(ui);
    } else {
        render_box(ui, style, render_control);
        add_default_bottom_spacing(ui, style, 2.0);
    }
}

fn tune_text_control_style(style: &StyleProps) -> StyleProps {
    let mut tuned = style.clone();
    let bg = tuned.bg.unwrap_or(egui::Color32::WHITE);
    let bg_too_dark = bg.r() <= 20 && bg.g() <= 20 && bg.b() <= 20;
    let bg_transparent = bg.a() <= 8;
    if bg_too_dark || bg_transparent {
        tuned.bg = Some(egui::Color32::WHITE);
    }

    if tuned.color.is_none() || tuned.color.is_some_and(|color| color.a() <= 8) {
        tuned.color = Some(egui::Color32::from_rgb(31, 31, 31));
    }
    if tuned.border_color.is_none() || tuned.border_color == Some(egui::Color32::TRANSPARENT) {
        tuned.border_color = Some(egui::Color32::from_rgb(143, 143, 143));
    }

    tuned
}

fn with_form_control_visuals<R>(
    ui: &mut egui::Ui,
    style: &StyleProps,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let fg_color = color_with_effective_opacity(
        style.color.unwrap_or(egui::Color32::from_rgb(31, 31, 31)),
        style,
    );
    let bg_color = color_with_effective_opacity(style.bg.unwrap_or(egui::Color32::WHITE), style);
    let border_color = color_with_effective_opacity(
        style
            .border_color
            .unwrap_or(egui::Color32::from_rgb(143, 143, 143)),
        style,
    );
    let corner_radius = style.border_radius.unwrap_or(0.0).clamp(0.0, 255.0).round() as u8;

    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        visuals.extreme_bg_color = bg_color;
        visuals.faint_bg_color = bg_color;
        visuals.code_bg_color = bg_color;
        visuals.widgets.inactive.bg_fill = bg_color;
        visuals.widgets.inactive.fg_stroke.color = fg_color;
        visuals.widgets.inactive.bg_stroke.color = border_color;
        visuals.widgets.noninteractive.bg_fill = bg_color;
        visuals.widgets.noninteractive.fg_stroke.color = fg_color;
        visuals.widgets.noninteractive.bg_stroke.color = border_color;
        visuals.widgets.noninteractive.corner_radius = corner_radius.into();
        visuals.widgets.hovered.bg_fill = bg_color;
        visuals.widgets.hovered.fg_stroke.color = fg_color;
        visuals.widgets.hovered.bg_stroke.color = border_color;
        visuals.widgets.hovered.corner_radius = corner_radius.into();
        visuals.widgets.active.bg_fill = bg_color;
        visuals.widgets.active.fg_stroke.color = fg_color;
        visuals.widgets.active.bg_stroke.color = border_color;
        visuals.widgets.active.corner_radius = corner_radius.into();
        visuals.widgets.inactive.corner_radius = corner_radius.into();
        visuals.override_text_color = Some(fg_color);
        add_contents(ui)
    })
    .inner
}

fn apply_html_alignment_attr(el: &HtmlElement, style: &StyleProps) -> StyleProps {
    let mut out = style.clone();
    if let Some(align) = attr(el, "align") {
        if let Some(text_align) = parse_html_align_value(align) {
            out.text_align = Some(text_align);
        }
    }
    out
}

fn parse_html_align_value(value: &str) -> Option<TextAlign> {
    match value.trim().to_ascii_lowercase().as_str() {
        "center" | "middle" => Some(TextAlign::Center),
        "right" | "end" => Some(TextAlign::Right),
        "left" | "start" => Some(TextAlign::Left),
        "justify" => Some(TextAlign::Justify),
        _ => None,
    }
}

fn parse_html_dimension(value: &str, reference: f32) -> Option<f32> {
    let raw = value.trim();
    if raw.is_empty() {
        return None;
    }

    if let Some(percent) = raw.strip_suffix('%') {
        let parsed = percent.trim().parse::<f32>().ok()?;
        return Some(reference * (parsed / 100.0));
    }

    parse_html_length(raw)
}

fn parse_html_length(value: &str) -> Option<f32> {
    parse_length(value).or_else(|| value.trim().parse::<f32>().ok())
}

fn add_default_bottom_spacing(ui: &mut egui::Ui, style: &StyleProps, fallback: f32) {
    if style.margin.bottom.is_none() && fallback > 0.0 {
        ui.add_space(fallback);
    }
}

fn build_rich_text(input: String, style: &StyleProps, effects: TextEffects) -> egui::RichText {
    let mut out = normalize_text_for_render(&input, style, &effects);
    let script = effects.script.or(style.script);
    if matches!(script, Some(ScriptPosition::Sub)) {
        out = format!("_{out}");
    } else if matches!(script, Some(ScriptPosition::Sup)) {
        out = format!("^{out}");
    }

    let mut rich = egui::RichText::new(out);

    let mut size = style.font_size;
    if effects.small {
        size = Some(size.unwrap_or(14.0) * 0.85);
    }
    if matches!(script, Some(ScriptPosition::Sub | ScriptPosition::Sup)) {
        size = Some(size.unwrap_or(14.0) * 0.8);
    }
    if let Some(v) = size {
        rich = rich.size(v);
    }
    if let Some(v) = style.color {
        rich = rich.color(color_with_effective_opacity(v, style));
    }
    let wants_bold = effects.strong || style.bold.unwrap_or(false);
    let wants_italic = effects.italics || style.italic.unwrap_or(false);
    if wants_italic {
        rich = rich.italics();
    }
    if effects.underline || style.underline.unwrap_or(false) {
        rich = rich.underline();
    }
    if effects.strike || style.strike.unwrap_or(false) {
        rich = rich.strikethrough();
    }
    rich = rich.family(select_text_font_family(style, effects, wants_bold, wants_italic));
    let mark_color = if effects.mark {
        style.bg.or(Some(egui::Color32::from_rgb(255, 241, 128)))
    } else {
        style.bg
    };
    if let Some(bg) = mark_color {
        rich = rich.background_color(color_with_effective_opacity(bg, style));
    }

    rich
}

fn select_text_font_family(
    style: &StyleProps,
    effects: TextEffects,
    wants_bold: bool,
    wants_italic: bool,
) -> egui::FontFamily {
    let prefers_mono = effects.mono || matches!(style.font_family, Some(FontFamilyChoice::Monospace));
    if prefers_mono {
        return egui::FontFamily::Name("pd-monospace".into());
    }

    match (wants_bold, wants_italic) {
        (true, true) => egui::FontFamily::Name("pd-proportional-bold-italic".into()),
        (true, false) => egui::FontFamily::Name("pd-proportional-bold".into()),
        (false, true) => egui::FontFamily::Name("pd-proportional-italic".into()),
        (false, false) => egui::FontFamily::Name("pd-proportional".into()),
    }
}

fn normalize_text_for_render(input: &str, style: &StyleProps, effects: &TextEffects) -> String {
    let mut out = match effective_white_space_mode(style, effects) {
        WhiteSpaceMode::Normal | WhiteSpaceMode::NoWrap => collapse_whitespace(input),
        WhiteSpaceMode::Pre | WhiteSpaceMode::PreWrap => input.to_owned(),
        WhiteSpaceMode::PreLine => collapse_whitespace_preserve_newlines(input),
    };

    match style.text_transform.unwrap_or(TextTransform::None) {
        TextTransform::None => {}
        TextTransform::Uppercase => out = out.to_uppercase(),
        TextTransform::Lowercase => out = out.to_lowercase(),
        TextTransform::Capitalize => out = capitalize_words(&out),
    }

    out
}

fn effective_white_space_mode(style: &StyleProps, effects: &TextEffects) -> WhiteSpaceMode {
    style.white_space.unwrap_or(if effects.mono {
        WhiteSpaceMode::Pre
    } else {
        WhiteSpaceMode::Normal
    })
}

fn collapse_whitespace_preserve_newlines(input: &str) -> String {
    let mut lines = Vec::new();
    for line in input.lines() {
        lines.push(collapse_whitespace(line));
    }
    lines.join("\n")
}

fn capitalize_words(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut capitalize_next = true;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            if capitalize_next {
                for up in ch.to_uppercase() {
                    out.push(up);
                }
                capitalize_next = false;
            } else {
                out.push(ch);
            }
        } else {
            capitalize_next = ch.is_whitespace();
            out.push(ch);
        }
    }
    out
}

fn effective_opacity(style: &StyleProps) -> f32 {
    let opacity = style.opacity.unwrap_or(1.0).clamp(0.0, 1.0);
    if opacity <= 0.001 && has_animation_hint(style) {
        1.0
    } else {
        opacity
    }
}

fn color_with_effective_opacity(color: egui::Color32, style: &StyleProps) -> egui::Color32 {
    let opacity = effective_opacity(style);
    if opacity >= 0.999 {
        return color;
    }
    let alpha = ((color.a() as f32) * opacity).round().clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha)
}

fn is_likely_screen_reader_only(style: &StyleProps) -> bool {
    let tiny_width = style.width.is_some_and(|value| value > 0.0 && value <= 2.0);
    let tiny_height = style
        .height
        .is_some_and(|value| value > 0.0 && value <= 2.0);
    let no_padding = style.padding.max_or(0.0) <= 0.0;
    let no_border = style.border_width.max_or(0.0) <= 0.0;

    // Common accessibility utility pattern:
    // width/height 1px + no border/padding.
    tiny_width && tiny_height && no_padding && no_border
}

fn style_suppresses_rendering(style: &StyleProps) -> bool {
    matches!(style.display, Some(Display::None))
        || matches!(style.visibility_hidden, Some(true))
        || effective_opacity(style) <= 0.001
}

fn has_animation_hint(style: &StyleProps) -> bool {
    let animation = style
        .raw_css
        .get("animation")
        .or_else(|| style.raw_css.get("animation-name"))
        .map(|value| value.trim().to_ascii_lowercase());
    match animation.as_deref() {
        Some("none") | Some("") => false,
        Some(_) => true,
        None => false,
    }
}

fn position_offset(style: &StyleProps) -> (f32, f32) {
    let horizontal = style
        .inset_left
        .or(style.inset_right.map(|value| -value))
        .unwrap_or(0.0);
    let vertical = style
        .inset_top
        .or(style.inset_bottom.map(|value| -value))
        .unwrap_or(0.0);
    (horizontal, vertical)
}

fn overflow_mode_x(style: &StyleProps) -> OverflowMode {
    style.overflow_x.unwrap_or(OverflowMode::Visible)
}

fn overflow_mode_y(style: &StyleProps) -> OverflowMode {
    style.overflow_y.unwrap_or(OverflowMode::Visible)
}

fn with_overflow_behavior(
    ui: &mut egui::Ui,
    style: &StyleProps,
    body: impl FnOnce(&mut egui::Ui),
) {
    let overflow_x = overflow_mode_x(style);
    let overflow_y = overflow_mode_y(style);
    let scroll_x = matches!(overflow_x, OverflowMode::Auto | OverflowMode::Scroll);
    let scroll_y = matches!(overflow_y, OverflowMode::Auto | OverflowMode::Scroll);
    let hidden_clip = matches!(overflow_x, OverflowMode::Hidden)
        || matches!(overflow_y, OverflowMode::Hidden);

    if scroll_x || scroll_y {
        let mut scroll_area = egui::ScrollArea::new([scroll_x, scroll_y]).auto_shrink([false, false]);
        if matches!(overflow_x, OverflowMode::Scroll) || matches!(overflow_y, OverflowMode::Scroll)
        {
            scroll_area = scroll_area.scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);
        }
        scroll_area.show(ui, body);
        return;
    }

    if hidden_clip {
        let clip_rect = ui.max_rect();
        ui.scope(|ui| {
            ui.shrink_clip_rect(clip_rect);
            body(ui);
        });
        return;
    }

    body(ui);
}

fn z_layer_order(style: &StyleProps) -> Option<egui::Order> {
    match style.z_index.unwrap_or(0).cmp(&0) {
        std::cmp::Ordering::Less => Some(egui::Order::Background),
        std::cmp::Ordering::Greater => Some(egui::Order::Foreground),
        std::cmp::Ordering::Equal => None,
    }
}

fn element_has_hidden_semantics(el: &HtmlElement) -> bool {
    attr(el, "hidden").is_some()
        || attr(el, "inert").is_some()
        || attr(el, "aria-hidden")
            .is_some_and(|value| value.eq_ignore_ascii_case("true") || value == "1")
}

fn render_box(ui: &mut egui::Ui, style: &StyleProps, body: impl FnOnce(&mut egui::Ui)) {
    let mut margin_top = style
        .margin
        .top
        .filter(|value| !value.is_infinite())
        .unwrap_or(0.0)
        .max(0.0);
    let (mut margin_right, margin_right_auto) = style.margin.right.map_or((0.0, false), |value| {
        if value.is_infinite() {
            (0.0, true)
        } else {
            (value.max(0.0), false)
        }
    });
    let margin_bottom = style
        .margin
        .bottom
        .filter(|value| !value.is_infinite())
        .unwrap_or(0.0)
        .max(0.0);
    let (mut margin_left, margin_left_auto) = style.margin.left.map_or((0.0, false), |value| {
        if value.is_infinite() {
            (0.0, true)
        } else {
            (value.max(0.0), false)
        }
    });
    let padding_top = style.padding.top_or(0.0).max(0.0);
    let padding_right = style.padding.right_or(0.0).max(0.0);
    let padding_bottom = style.padding.bottom_or(0.0).max(0.0);
    let padding_left = style.padding.left_or(0.0).max(0.0);
    let border_top = style.border_width.top_or(0.0).max(0.0);
    let border_right = style.border_width.right_or(0.0).max(0.0);
    let border_bottom = style.border_width.bottom_or(0.0).max(0.0);
    let border_left = style.border_width.left_or(0.0).max(0.0);
    let border_color = color_with_effective_opacity(
        style.border_color.unwrap_or(egui::Color32::GRAY),
        style,
    );
    let border_radius = style.border_radius.unwrap_or(0.0).clamp(0.0, 255.0);

    if !matches!(style.position.unwrap_or(PositionMode::Static), PositionMode::Static) {
        let (offset_x, offset_y) = position_offset(style);
        margin_left = (margin_left + offset_x).max(0.0);
        margin_top = (margin_top + offset_y).max(0.0);
    }

    if margin_top > 0.0 {
        ui.add_space(margin_top);
    }

    let viewport_width = ui.available_width().max(1.0);
    let inline_layout = matches!(style.display, Some(Display::Inline));
    let specified_width = style.width.or_else(|| {
        style
            .width_percent
            .map(|percent| viewport_width * (percent / 100.0))
    });
    let mut width = if inline_layout {
        specified_width.unwrap_or(0.0).max(0.0)
    } else {
        specified_width
            .unwrap_or((viewport_width - margin_left - margin_right).max(1.0))
            .max(1.0)
    };
    if !inline_layout && let Some(min_width) = style.min_width {
        width = width.max(min_width.max(0.0));
    }
    if !inline_layout && let Some(max_width) = style.max_width {
        width = width.min(max_width.max(1.0));
    }

    if !inline_layout && specified_width.is_none() {
        width = (viewport_width - margin_left - margin_right).max(1.0);
        if let Some(min_width) = style.min_width {
            width = width.max(min_width.max(0.0));
        }
        if let Some(max_width) = style.max_width {
            width = width.min(max_width.max(1.0));
        }
    }
    if (margin_left_auto || margin_right_auto) && (!inline_layout || width > 0.0) {
        let remaining = (viewport_width - width - margin_left - margin_right).max(0.0);
        match (margin_left_auto, margin_right_auto) {
            (true, true) => {
                let side = remaining / 2.0;
                margin_left = side;
                margin_right = side;
            }
            (true, false) => margin_left = remaining,
            (false, true) => margin_right = remaining,
            (false, false) => {}
        }
    }

    let mut height = if inline_layout {
        0.0
    } else {
        style.height.unwrap_or(0.0).max(0.0)
    };
    if !inline_layout {
        if let Some(min_height) = style.min_height {
            height = height.max(min_height.max(0.0));
        }
        if let Some(max_height) = style.max_height {
            height = height.min(max_height.max(0.0));
        }
    }

    let mut frame = egui::Frame::NONE;
    if let Some(bg) = style.bg {
        frame = frame.fill(color_with_effective_opacity(bg, style));
    }
    if border_radius > 0.0 {
        frame = frame.corner_radius(border_radius.round() as u8);
    }
    let mut body = Some(body);

    if inline_layout && specified_width.is_none() && height <= 0.0 {
        if margin_left > 0.0 {
            ui.add_space(margin_left);
        }

        let response = frame
            .inner_margin(egui::Margin {
                left: margin_component(padding_left + border_left),
                right: margin_component(padding_right + border_right),
                top: margin_component(padding_top + border_top),
                bottom: margin_component(padding_bottom + border_bottom),
            })
            .show(ui, |ui| {
                with_overflow_behavior(
                    ui,
                    style,
                    body.take().expect("render_box body should run exactly once"),
                );
            })
            .response;

        paint_box_border(
            ui.painter(),
            response.rect,
            border_top,
            border_right,
            border_bottom,
            border_left,
            border_color,
            border_radius,
        );

        if margin_right > 0.0 {
            ui.add_space(margin_right);
        }
        if margin_bottom > 0.0 {
            ui.add_space(margin_bottom);
        }
        return;
    }

    let horizontal_align = match style.text_align.unwrap_or(TextAlign::Left) {
        TextAlign::Left | TextAlign::Justify => egui::Align::Min,
        TextAlign::Center => egui::Align::Center,
        TextAlign::Right => egui::Align::Max,
    };

    let mut paint = |ui: &mut egui::Ui| {
        let mut content_rect: Option<egui::Rect> = None;
        ui.horizontal(|ui| {
            if margin_left > 0.0 {
                ui.add_space(margin_left);
            }

            let mut render_inner = |ui: &mut egui::Ui| {
                frame.show(ui, |ui| {
                    if border_top > 0.0 {
                        ui.add_space(border_top);
                    }
                    if padding_top > 0.0 {
                        ui.add_space(padding_top);
                    }

                    ui.horizontal(|ui| {
                        if border_left > 0.0 {
                            ui.add_space(border_left);
                        }
                        if padding_left > 0.0 {
                            ui.add_space(padding_left);
                        }

                        ui.vertical(|ui| {
                            with_overflow_behavior(
                                ui,
                                style,
                                body.take().expect("render_box body should run exactly once"),
                            );
                        });

                        if padding_right > 0.0 {
                            ui.add_space(padding_right);
                        }
                        if border_right > 0.0 {
                            ui.add_space(border_right);
                        }
                    });

                    if padding_bottom > 0.0 {
                        ui.add_space(padding_bottom);
                    }
                    if border_bottom > 0.0 {
                        ui.add_space(border_bottom);
                    }
                })
                .response
            };

            let response = if inline_layout && width <= 0.0 && height <= 0.0 {
                render_inner(ui)
            } else {
                ui.allocate_ui_with_layout(
                    egui::vec2(width.max(1.0), height),
                    egui::Layout::top_down(horizontal_align),
                    |ui| {
                        let _ = render_inner(ui);
                    },
                )
                .response
            };
            content_rect = Some(response.rect);

            if margin_right > 0.0 {
                ui.add_space(margin_right);
            }
        });

        if let Some(rect) = content_rect {
            paint_box_border(
                ui.painter(),
                rect,
                border_top,
                border_right,
                border_bottom,
                border_left,
                border_color,
                border_radius,
            );
        }
    };

    if let Some(order) = z_layer_order(style) {
        let layer_id = egui::LayerId::new(order, ui.id().with(("css-z", style.z_index, ui.next_auto_id())));
        ui.scope_builder(egui::UiBuilder::new().layer_id(layer_id), paint);
    } else {
        paint(ui);
    }

    if margin_bottom > 0.0 {
        ui.add_space(margin_bottom);
    }
}

fn margin_component(value: f32) -> i8 {
    value.round().clamp(0.0, 127.0) as i8
}

fn paint_box_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
    color: egui::Color32,
    border_radius: f32,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let top = top.clamp(0.0, rect.height());
    let right = right.clamp(0.0, rect.width());
    let bottom = bottom.clamp(0.0, rect.height());
    let left = left.clamp(0.0, rect.width());

    let symmetric_border = top > 0.0
        && (top - right).abs() <= 0.001
        && (top - bottom).abs() <= 0.001
        && (top - left).abs() <= 0.001;
    if border_radius > 0.0 && symmetric_border {
        painter.rect_stroke(
            rect,
            border_radius.round() as u8,
            egui::Stroke::new(top, color),
            egui::StrokeKind::Middle,
        );
        return;
    }

    if top > 0.0 {
        let top_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.min.y + top));
        painter.rect_filled(top_rect, 0.0, color);
    }
    if right > 0.0 {
        let right_rect =
            egui::Rect::from_min_max(egui::pos2(rect.max.x - right, rect.min.y), rect.max);
        painter.rect_filled(right_rect, 0.0, color);
    }
    if bottom > 0.0 {
        let bottom_rect =
            egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - bottom), rect.max);
        painter.rect_filled(bottom_rect, 0.0, color);
    }
    if left > 0.0 {
        let left_rect =
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + left, rect.max.y));
        painter.rect_filled(left_rect, 0.0, color);
    }
}

fn render_flex(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let direction = style.flex_direction.unwrap_or(FlexDirection::Row);
    let flex_wrap = style.flex_wrap.unwrap_or(FlexWrap::NoWrap);
    let align_items = style.align_items.unwrap_or(AlignItems::Start);
    let align_content = style.align_content.unwrap_or(AlignContent::Start);
    let justify_content = style.justify_content.unwrap_or(JustifyContent::Start);
    let gap = style.gap.unwrap_or(0.0).max(0.0);

    let mut cross_align = match align_items {
        AlignItems::Start | AlignItems::Stretch => egui::Align::Min,
        AlignItems::Center => egui::Align::Center,
        AlignItems::End => egui::Align::Max,
    };
    if matches!(flex_wrap, FlexWrap::Wrap) {
        cross_align = match align_content {
            AlignContent::Start => egui::Align::Min,
            AlignContent::Center => egui::Align::Center,
            AlignContent::End => egui::Align::Max,
            AlignContent::SpaceBetween
            | AlignContent::SpaceAround
            | AlignContent::SpaceEvenly
            | AlignContent::Stretch => cross_align,
        };
    }

    let main_align = match justify_content {
        JustifyContent::Start
        | JustifyContent::SpaceBetween
        | JustifyContent::SpaceAround
        | JustifyContent::SpaceEvenly => egui::Align::Min,
        JustifyContent::Center => egui::Align::Center,
        JustifyContent::End => egui::Align::Max,
    };

    let mut layout = match direction {
        FlexDirection::Row => egui::Layout::left_to_right(cross_align),
        FlexDirection::Column => egui::Layout::top_down(cross_align),
    };
    layout = layout.with_main_align(main_align);
    layout = layout.with_main_wrap(matches!(flex_wrap, FlexWrap::Wrap));
    if matches!(align_items, AlignItems::Stretch) {
        layout = layout.with_cross_justify(true);
    }
    if matches!(
        align_content,
        AlignContent::SpaceBetween
            | AlignContent::SpaceAround
            | AlignContent::SpaceEvenly
            | AlignContent::Stretch
    ) {
        layout = layout.with_cross_justify(true);
    }
    if matches!(
        justify_content,
        JustifyContent::SpaceBetween | JustifyContent::SpaceAround | JustifyContent::SpaceEvenly
    ) {
        layout = layout.with_main_justify(true);
    }

    render_box(ui, style, |ui| {
        ui.with_layout(layout, |ui| {
            let mut first = true;
            for child in &el.children {
                if !first && gap > 0.0 {
                    ui.add_space(gap);
                }
                first = false;
                render_node(ui, child, ctx, style);
            }
        });
    });
}

fn render_grid(ui: &mut egui::Ui, el: &HtmlElement, ctx: &mut Ctx<'_>, style: &StyleProps) {
    let gap = style.gap.unwrap_or(0.0).max(0.0);
    let min_track_width = style
        .raw_css
        .get("grid-template-columns")
        .or_else(|| style.raw_css.get("grid-template"))
        .and_then(|value| parse_grid_min_track_width(value))
        .unwrap_or(220.0)
        .max(1.0);

    render_box(ui, style, |ui| {
        let children = el
            .children
            .iter()
            .filter(|node| !matches!(node, HtmlNode::Text(text) if text.trim().is_empty()))
            .collect::<Vec<_>>();
        if children.is_empty() {
            return;
        }

        let available = ui.available_width().max(1.0);
        let mut columns = ((available + gap) / (min_track_width + gap).max(1.0)).floor() as usize;
        columns = columns.max(1).min(children.len());

        let original_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = egui::vec2(gap, gap);
        ui.columns(columns, |cols| {
            for (index, child) in children.iter().enumerate() {
                let col = index % columns;
                render_node(&mut cols[col], child, ctx, style);
            }
        });
        ui.spacing_mut().item_spacing = original_spacing;
    });
}

fn parse_grid_min_track_width(value: &str) -> Option<f32> {
    let lower = value.to_ascii_lowercase();
    if let Some(start) = lower.find("minmax(") {
        let rest = &value[(start + "minmax(".len())..];
        if let Some(end) = rest.find(')')
            && let Some(first) = rest[..end].split(',').next()
            && let Some(parsed) = parse_non_negative_length(first.trim())
        {
            return Some(parsed.max(1.0));
        }
    }

    for token in value.split(|ch: char| ch.is_whitespace() || ch == ',') {
        if let Some(parsed) = parse_non_negative_length(token.trim()) {
            return Some(parsed.max(1.0));
        }
    }

    None
}

fn apply_html_presentational_attributes(el: &HtmlElement, style: &mut StyleProps) {
    if style.text_align.is_none()
        && let Some(align) = attr(el, "align").and_then(parse_html_align_value)
    {
        style.text_align = Some(align);
    }

    if style.bg.is_none()
        && let Some(bg) = attr(el, "bgcolor").and_then(parse_color)
    {
        style.bg = Some(bg);
    }

    if style.color.is_none()
        && let Some(color) = attr(el, "color")
            .or_else(|| attr(el, "text"))
            .and_then(parse_color)
    {
        style.color = Some(color);
    }

    if style.border_color.is_none()
        && let Some(border_color) = attr(el, "bordercolor").and_then(parse_color)
    {
        style.border_color = Some(border_color);
    }

    if style.border_width.max_or(0.0) <= 0.0
        && let Some(border_width) = attr(el, "border")
            .and_then(parse_html_length)
            .map(|value| value.max(0.0))
    {
        style.border_width = Edges::all(border_width);
    }

    if style.width.is_none()
        && style.width_percent.is_none()
        && let Some(width) = attr(el, "width")
    {
        if let Some(percent) = parse_percentage(width) {
            style.width_percent = Some(percent);
        } else if let Some(px) = parse_html_length(width) {
            style.width = Some(px.max(1.0));
        }
    }

    if style.height.is_none()
        && let Some(height) = attr(el, "height")
            .and_then(parse_html_length)
            .map(|value| value.max(1.0))
    {
        style.height = Some(height);
    }

    if style.font_family.is_none()
        && let Some(font_family) = attr(el, "face").and_then(parse_font_family)
    {
        style.font_family = Some(font_family);
    }

    if style.font_size.is_none()
        && let Some(font_size) = attr(el, "size").and_then(parse_legacy_font_size)
    {
        style.font_size = Some(font_size);
    }
}

fn parse_legacy_font_size(value: &str) -> Option<f32> {
    let raw = value.trim();
    if raw.is_empty() {
        return None;
    }

    if let Some(relative) = raw.strip_prefix('+') {
        let delta = relative.trim().parse::<i32>().ok()?;
        let resolved = (3 + delta).clamp(1, 7);
        return Some(legacy_font_size_px(resolved));
    }

    if let Some(relative) = raw.strip_prefix('-') {
        let delta = relative.trim().parse::<i32>().ok()?;
        let resolved = (3 - delta).clamp(1, 7);
        return Some(legacy_font_size_px(resolved));
    }

    let absolute = raw.parse::<i32>().ok()?.clamp(1, 7);
    Some(legacy_font_size_px(absolute))
}

fn legacy_font_size_px(value: i32) -> f32 {
    match value {
        1 => 10.0,
        2 => 13.0,
        3 => 16.0,
        4 => 18.0,
        5 => 24.0,
        6 => 32.0,
        _ => 48.0,
    }
}

fn style_for(
    el: &HtmlElement,
    sheet: &StyleSheet,
    inherited: &StyleProps,
    ancestors: &[SelectorSubject],
) -> StyleProps {
    let mut style = StyleProps::default();
    let mut priorities = StylePriority::default();

    for rule in &sheet.rules {
        if matches_selector(&rule.sel, el, ancestors) {
            for declaration in &rule.declarations {
                apply_declaration_with_cascade(
                    declaration,
                    CascadePriority {
                        important: declaration.important,
                        specificity: rule.specificity,
                        source_order: declaration.source_order,
                    },
                    &mut style,
                    &mut priorities,
                );
            }
        }
    }

    if let Some(inline) = attr(el, "style") {
        let mut declaration_order = usize::MAX / 4;
        for declaration in parse_declaration_entries(inline, &mut declaration_order) {
            apply_declaration_with_cascade(
                &declaration,
                CascadePriority {
                    important: declaration.important,
                    specificity: 1000,
                    source_order: declaration.source_order,
                },
                &mut style,
                &mut priorities,
            );
        }
    }

    apply_raw_css_aliases(&mut style);
    apply_html_presentational_attributes(el, &mut style);

    if style.color.is_none() {
        style.color = inherited.color;
    }
    if style.font_size.is_none() {
        style.font_size = inherited.font_size;
    }
    if style.visibility_hidden.is_none() {
        style.visibility_hidden = inherited.visibility_hidden;
    }
    if style.text_align.is_none() {
        style.text_align = inherited.text_align;
    }
    if style.font_family.is_none() {
        style.font_family = inherited.font_family;
    }
    if style.bold.is_none() {
        style.bold = inherited.bold;
    }
    if style.italic.is_none() {
        style.italic = inherited.italic;
    }
    if style.underline.is_none() {
        style.underline = inherited.underline;
    }
    if style.strike.is_none() {
        style.strike = inherited.strike;
    }
    if style.script.is_none() {
        style.script = inherited.script;
    }
    if style.line_height.is_none() {
        style.line_height = inherited.line_height;
    }
    if style.text_transform.is_none() {
        style.text_transform = inherited.text_transform;
    }
    if style.white_space.is_none() {
        style.white_space = inherited.white_space;
    }
    if style.list_style_type.is_none() {
        style.list_style_type = inherited.list_style_type.clone();
    }
    let inherited_effective_opacity = if inherited.opacity.is_some() {
        Some(effective_opacity(inherited))
    } else {
        None
    };
    if let Some(parent_opacity) = inherited_effective_opacity {
        let own_opacity = style.opacity.unwrap_or(1.0);
        style.opacity = Some((parent_opacity * own_opacity).clamp(0.0, 1.0));
    } else if let Some(own_opacity) = style.opacity {
        style.opacity = Some(own_opacity.clamp(0.0, 1.0));
    }
    if style.text_align.is_none() {
        if let Some(dir) = attr(el, "dir") {
            if dir.eq_ignore_ascii_case("rtl") {
                style.text_align = Some(TextAlign::Right);
            } else if dir.eq_ignore_ascii_case("ltr") {
                style.text_align = Some(TextAlign::Left);
            }
        }
    }

    style
}

fn apply_declaration_with_cascade(
    declaration: &CssDeclaration,
    priority: CascadePriority,
    style: &mut StyleProps,
    priorities: &mut StylePriority,
) {
    if declaration.value.eq_ignore_ascii_case("inherit") {
        let _ = apply_inherit_keyword(&declaration.name, priority, style, priorities);
    } else {
        apply_style_with_priority(&declaration.parsed, priority, style, priorities);
    }

    apply_raw_css_property_with_priority(
        declaration.name.as_str(),
        declaration.value.as_str(),
        priority,
        style,
        priorities,
    );
}

fn apply_inherit_keyword(
    property_name: &str,
    priority: CascadePriority,
    style: &mut StyleProps,
    priorities: &mut StylePriority,
) -> bool {
    match property_name {
        "color" => {
            apply_cascade_value(&mut style.color, &mut priorities.color, None, priority);
            true
        }
        "font-size" => {
            apply_cascade_value(
                &mut style.font_size,
                &mut priorities.font_size,
                None,
                priority,
            );
            true
        }
        "text-align" => {
            apply_cascade_value(
                &mut style.text_align,
                &mut priorities.text_align,
                None,
                priority,
            );
            true
        }
        "font-family" => {
            apply_cascade_value(
                &mut style.font_family,
                &mut priorities.font_family,
                None,
                priority,
            );
            true
        }
        "font-weight" => {
            apply_cascade_value(&mut style.bold, &mut priorities.bold, None, priority);
            true
        }
        "font-style" => {
            apply_cascade_value(&mut style.italic, &mut priorities.italic, None, priority);
            true
        }
        "text-decoration" | "text-decoration-line" => {
            apply_cascade_value(
                &mut style.underline,
                &mut priorities.underline,
                None,
                priority,
            );
            apply_cascade_value(&mut style.strike, &mut priorities.strike, None, priority);
            true
        }
        "vertical-align" => {
            apply_cascade_value(&mut style.script, &mut priorities.script, None, priority);
            true
        }
        "line-height" => {
            apply_cascade_value(
                &mut style.line_height,
                &mut priorities.line_height,
                None,
                priority,
            );
            true
        }
        "text-transform" => {
            apply_cascade_value(
                &mut style.text_transform,
                &mut priorities.text_transform,
                None,
                priority,
            );
            true
        }
        "white-space" => {
            apply_cascade_value(
                &mut style.white_space,
                &mut priorities.white_space,
                None,
                priority,
            );
            true
        }
        "list-style" | "list-style-type" => {
            apply_cascade_value(
                &mut style.list_style_type,
                &mut priorities.list_style_type,
                None,
                priority,
            );
            true
        }
        _ => false,
    }
}

fn apply_style_with_priority(
    incoming: &StyleProps,
    priority: CascadePriority,
    style: &mut StyleProps,
    priorities: &mut StylePriority,
) {
    if incoming.display.is_some() {
        apply_cascade_value(
            &mut style.display,
            &mut priorities.display,
            incoming.display,
            priority,
        );
    }
    if incoming.visibility_hidden.is_some() {
        apply_cascade_value(
            &mut style.visibility_hidden,
            &mut priorities.visibility_hidden,
            incoming.visibility_hidden,
            priority,
        );
    }
    if incoming.opacity.is_some() {
        apply_cascade_value(
            &mut style.opacity,
            &mut priorities.opacity,
            incoming.opacity,
            priority,
        );
    }
    if incoming.text_align.is_some() {
        apply_cascade_value(
            &mut style.text_align,
            &mut priorities.text_align,
            incoming.text_align,
            priority,
        );
    }
    if incoming.font_family.is_some() {
        apply_cascade_value(
            &mut style.font_family,
            &mut priorities.font_family,
            incoming.font_family,
            priority,
        );
    }
    if incoming.color.is_some() {
        apply_cascade_value(
            &mut style.color,
            &mut priorities.color,
            incoming.color,
            priority,
        );
    }
    if incoming.bg.is_some() {
        apply_cascade_value(&mut style.bg, &mut priorities.bg, incoming.bg, priority);
    }
    if incoming.font_size.is_some() {
        apply_cascade_value(
            &mut style.font_size,
            &mut priorities.font_size,
            incoming.font_size,
            priority,
        );
    }
    if incoming.bold.is_some() {
        apply_cascade_value(
            &mut style.bold,
            &mut priorities.bold,
            incoming.bold,
            priority,
        );
    }
    if incoming.italic.is_some() {
        apply_cascade_value(
            &mut style.italic,
            &mut priorities.italic,
            incoming.italic,
            priority,
        );
    }
    if incoming.underline.is_some() {
        apply_cascade_value(
            &mut style.underline,
            &mut priorities.underline,
            incoming.underline,
            priority,
        );
    }
    if incoming.strike.is_some() {
        apply_cascade_value(
            &mut style.strike,
            &mut priorities.strike,
            incoming.strike,
            priority,
        );
    }
    if incoming.script.is_some() {
        apply_cascade_value(
            &mut style.script,
            &mut priorities.script,
            incoming.script,
            priority,
        );
    }
    if incoming.line_height.is_some() {
        apply_cascade_value(
            &mut style.line_height,
            &mut priorities.line_height,
            incoming.line_height,
            priority,
        );
    }
    if incoming.flex_direction.is_some() {
        apply_cascade_value(
            &mut style.flex_direction,
            &mut priorities.flex_direction,
            incoming.flex_direction,
            priority,
        );
    }
    if incoming.flex_wrap.is_some() {
        apply_cascade_value(
            &mut style.flex_wrap,
            &mut priorities.flex_wrap,
            incoming.flex_wrap,
            priority,
        );
    }
    if incoming.justify_content.is_some() {
        apply_cascade_value(
            &mut style.justify_content,
            &mut priorities.justify_content,
            incoming.justify_content,
            priority,
        );
    }
    if incoming.align_items.is_some() {
        apply_cascade_value(
            &mut style.align_items,
            &mut priorities.align_items,
            incoming.align_items,
            priority,
        );
    }
    if incoming.align_content.is_some() {
        apply_cascade_value(
            &mut style.align_content,
            &mut priorities.align_content,
            incoming.align_content,
            priority,
        );
    }
    if incoming.gap.is_some() {
        apply_cascade_value(&mut style.gap, &mut priorities.gap, incoming.gap, priority);
    }
    if incoming.text_transform.is_some() {
        apply_cascade_value(
            &mut style.text_transform,
            &mut priorities.text_transform,
            incoming.text_transform,
            priority,
        );
    }
    if incoming.white_space.is_some() {
        apply_cascade_value(
            &mut style.white_space,
            &mut priorities.white_space,
            incoming.white_space,
            priority,
        );
    }
    if incoming.list_style_type.is_some() {
        apply_cascade_value(
            &mut style.list_style_type,
            &mut priorities.list_style_type,
            incoming.list_style_type.clone(),
            priority,
        );
    }
    if incoming.position.is_some() {
        apply_cascade_value(
            &mut style.position,
            &mut priorities.position,
            incoming.position,
            priority,
        );
    }
    if incoming.inset_top.is_some() {
        apply_cascade_value(
            &mut style.inset_top,
            &mut priorities.inset_top,
            incoming.inset_top,
            priority,
        );
    }
    if incoming.inset_right.is_some() {
        apply_cascade_value(
            &mut style.inset_right,
            &mut priorities.inset_right,
            incoming.inset_right,
            priority,
        );
    }
    if incoming.inset_bottom.is_some() {
        apply_cascade_value(
            &mut style.inset_bottom,
            &mut priorities.inset_bottom,
            incoming.inset_bottom,
            priority,
        );
    }
    if incoming.inset_left.is_some() {
        apply_cascade_value(
            &mut style.inset_left,
            &mut priorities.inset_left,
            incoming.inset_left,
            priority,
        );
    }
    if incoming.z_index.is_some() {
        apply_cascade_value(
            &mut style.z_index,
            &mut priorities.z_index,
            incoming.z_index,
            priority,
        );
    }
    if incoming.overflow_x.is_some() {
        apply_cascade_value(
            &mut style.overflow_x,
            &mut priorities.overflow_x,
            incoming.overflow_x,
            priority,
        );
    }
    if incoming.overflow_y.is_some() {
        apply_cascade_value(
            &mut style.overflow_y,
            &mut priorities.overflow_y,
            incoming.overflow_y,
            priority,
        );
    }
    if incoming.width.is_some() {
        apply_cascade_value(
            &mut style.width,
            &mut priorities.width,
            incoming.width,
            priority,
        );
    }
    if incoming.width_percent.is_some() {
        apply_cascade_value(
            &mut style.width_percent,
            &mut priorities.width_percent,
            incoming.width_percent,
            priority,
        );
    }
    if incoming.min_width.is_some() {
        apply_cascade_value(
            &mut style.min_width,
            &mut priorities.min_width,
            incoming.min_width,
            priority,
        );
    }
    if incoming.max_width.is_some() {
        apply_cascade_value(
            &mut style.max_width,
            &mut priorities.max_width,
            incoming.max_width,
            priority,
        );
    }
    if incoming.height.is_some() {
        apply_cascade_value(
            &mut style.height,
            &mut priorities.height,
            incoming.height,
            priority,
        );
    }
    if incoming.min_height.is_some() {
        apply_cascade_value(
            &mut style.min_height,
            &mut priorities.min_height,
            incoming.min_height,
            priority,
        );
    }
    if incoming.max_height.is_some() {
        apply_cascade_value(
            &mut style.max_height,
            &mut priorities.max_height,
            incoming.max_height,
            priority,
        );
    }
    apply_edge_with_priority(
        incoming.margin.top,
        priority,
        &mut style.margin.top,
        &mut priorities.margin.top,
    );
    apply_edge_with_priority(
        incoming.margin.right,
        priority,
        &mut style.margin.right,
        &mut priorities.margin.right,
    );
    apply_edge_with_priority(
        incoming.margin.bottom,
        priority,
        &mut style.margin.bottom,
        &mut priorities.margin.bottom,
    );
    apply_edge_with_priority(
        incoming.margin.left,
        priority,
        &mut style.margin.left,
        &mut priorities.margin.left,
    );
    apply_edge_with_priority(
        incoming.padding.top,
        priority,
        &mut style.padding.top,
        &mut priorities.padding.top,
    );
    apply_edge_with_priority(
        incoming.padding.right,
        priority,
        &mut style.padding.right,
        &mut priorities.padding.right,
    );
    apply_edge_with_priority(
        incoming.padding.bottom,
        priority,
        &mut style.padding.bottom,
        &mut priorities.padding.bottom,
    );
    apply_edge_with_priority(
        incoming.padding.left,
        priority,
        &mut style.padding.left,
        &mut priorities.padding.left,
    );
    apply_edge_with_priority(
        incoming.border_width.top,
        priority,
        &mut style.border_width.top,
        &mut priorities.border_width.top,
    );
    apply_edge_with_priority(
        incoming.border_width.right,
        priority,
        &mut style.border_width.right,
        &mut priorities.border_width.right,
    );
    apply_edge_with_priority(
        incoming.border_width.bottom,
        priority,
        &mut style.border_width.bottom,
        &mut priorities.border_width.bottom,
    );
    apply_edge_with_priority(
        incoming.border_width.left,
        priority,
        &mut style.border_width.left,
        &mut priorities.border_width.left,
    );
    if incoming.border_color.is_some() {
        apply_cascade_value(
            &mut style.border_color,
            &mut priorities.border_color,
            incoming.border_color,
            priority,
        );
    }
    if incoming.border_radius.is_some() {
        apply_cascade_value(
            &mut style.border_radius,
            &mut priorities.border_radius,
            incoming.border_radius,
            priority,
        );
    }
}

fn apply_raw_css_property_with_priority(
    name: &str,
    value: &str,
    priority: CascadePriority,
    style: &mut StyleProps,
    priorities: &mut StylePriority,
) {
    let existing = priorities.raw_css.get(name).copied();
    if should_apply_priority(priority, existing) {
        style.raw_css.insert(name.to_owned(), value.to_owned());
        priorities.raw_css.insert(name.to_owned(), priority);
    }
}

fn apply_edge_with_priority(
    incoming: Option<f32>,
    priority: CascadePriority,
    target: &mut Option<f32>,
    target_priority: &mut Option<CascadePriority>,
) {
    if incoming.is_some() {
        apply_cascade_value(target, target_priority, incoming, priority);
    }
}

fn apply_cascade_value<T>(
    target: &mut Option<T>,
    target_priority: &mut Option<CascadePriority>,
    incoming: Option<T>,
    priority: CascadePriority,
) {
    if should_apply_priority(priority, *target_priority) {
        *target = incoming;
        *target_priority = Some(priority);
    }
}

fn should_apply_priority(incoming: CascadePriority, existing: Option<CascadePriority>) -> bool {
    match existing {
        Some(current) => incoming >= current,
        None => true,
    }
}

fn extract_styles(root: &HtmlElement) -> StyleSheet {
    let mut css = String::new();
    collect_style_source(&root.children, false, &mut css);
    StyleSheet {
        rules: parse_css_rules(&css),
    }
}

fn collect_style_source(nodes: &[HtmlNode], inside_noscript: bool, out: &mut String) {
    for node in nodes {
        let HtmlNode::Element(el) = node else {
            continue;
        };

        if canonical_element_tag(el.tag.as_str()) == "template" {
            continue;
        }

        let next_inside_noscript = inside_noscript || el.tag == "noscript";
        if next_inside_noscript {
            continue;
        }

        if el.tag == "style" {
            out.push_str(&collect_text(&el.children));
            out.push('\n');
        }
        collect_style_source(&el.children, next_inside_noscript, out);
    }
}

fn count_style_tags(nodes: &[HtmlNode]) -> usize {
    count_style_tags_with_context(nodes, false)
}

fn count_style_tags_with_context(nodes: &[HtmlNode], inside_noscript: bool) -> usize {
    let mut count = 0_usize;
    for node in nodes {
        let HtmlNode::Element(el) = node else {
            continue;
        };

        if canonical_element_tag(el.tag.as_str()) == "template" {
            continue;
        }

        let next_inside_noscript = inside_noscript || el.tag == "noscript";
        if next_inside_noscript {
            continue;
        }

        if el.tag == "style" {
            count = count.saturating_add(1);
        }
        count = count.saturating_add(count_style_tags_with_context(
            &el.children,
            next_inside_noscript,
        ));
    }
    count
}

fn collect_script_descriptors(nodes: &[HtmlNode], base_url: &str, out: &mut Vec<ScriptDescriptor>) {
    for node in nodes {
        let HtmlNode::Element(el) = node else {
            continue;
        };

        if canonical_element_tag(el.tag.as_str()) == "template" {
            continue;
        }

        if el.tag == "script" && script_tag_is_executable(el) {
            if let Some(src) = attr(el, "src").and_then(|value| resolve_link(base_url, value)) {
                out.push(ScriptDescriptor::External { url: src });
            } else {
                let source = collect_text(&el.children);
                if !source.trim().is_empty() {
                    out.push(ScriptDescriptor::Inline { source });
                }
            }
        }

        collect_script_descriptors(&el.children, base_url, out);
    }
}

fn collect_id_elements(nodes: &[HtmlNode], max_elements: usize, out: &mut Vec<IdElementSnapshot>) {
    if out.len() >= max_elements {
        return;
    }

    for node in nodes {
        if out.len() >= max_elements {
            return;
        }

        let HtmlNode::Element(el) = node else {
            continue;
        };

        if let Some(id) = attr(el, "id") {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                out.push(IdElementSnapshot {
                    id: trimmed.to_owned(),
                    tag_name: el.tag.to_ascii_uppercase(),
                    text_content: collapse_whitespace(&collect_text(&el.children)),
                    attributes: el.attrs.clone(),
                });
            }
        }

        collect_id_elements(&el.children, max_elements, out);
    }
}

fn collect_subresources_from_nodes(
    nodes: &[HtmlNode],
    base_url: &str,
    stylesheets: &mut HashSet<String>,
    images: &mut HashSet<String>,
    scripts: &mut HashSet<String>,
) {
    for node in nodes {
        let HtmlNode::Element(el) = node else {
            continue;
        };

        if canonical_element_tag(el.tag.as_str()) == "template" {
            continue;
        }

        let tag = canonical_element_tag(el.tag.as_str());
        match tag {
            "img" => {
                if let Some(src) =
                    image_source_attr(el).and_then(|value| resolve_link(base_url, value))
                {
                    images.insert(src);
                }
            }
            "source" => {
                if let Some(src) =
                    image_source_attr(el).and_then(|value| resolve_link(base_url, value))
                {
                    images.insert(src);
                }
            }
            "link" => {
                if is_stylesheet_link(el) {
                    if let Some(href) =
                        attr(el, "href").and_then(|value| resolve_link(base_url, value))
                    {
                        stylesheets.insert(href);
                    }
                }
            }
            "script" => {
                if script_tag_is_executable(el)
                    && let Some(src) =
                        attr(el, "src").and_then(|value| resolve_link(base_url, value))
                {
                    scripts.insert(src);
                }
            }
            "video" => {
                if let Some(poster) =
                    attr(el, "poster").and_then(|value| resolve_link(base_url, value))
                {
                    images.insert(poster);
                }
            }
            _ => {}
        }

        if let Some(background) =
            parse_background_resource_attr(el).and_then(|value| resolve_link(base_url, value))
        {
            images.insert(background);
        }

        collect_subresources_from_nodes(&el.children, base_url, stylesheets, images, scripts);
    }
}

fn image_source_attr<'a>(el: &'a HtmlElement) -> Option<&'a str> {
    attr(el, "src")
        .filter(|value| !value.trim().is_empty())
        .or_else(|| attr(el, "data-src").filter(|value| !value.trim().is_empty()))
        .or_else(|| attr(el, "srcset").and_then(parse_srcset_first_url))
        .or_else(|| attr(el, "data-srcset").and_then(parse_srcset_first_url))
}

fn parse_srcset_first_url(srcset: &str) -> Option<&str> {
    let first = srcset.split(',').next()?.trim();
    if first.is_empty() {
        return None;
    }

    let url = first
        .split_ascii_whitespace()
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    Some(url)
}

fn is_stylesheet_link(el: &HtmlElement) -> bool {
    attr(el, "rel")
        .map(|value| {
            value
                .split_ascii_whitespace()
                .any(|token| token.eq_ignore_ascii_case("stylesheet"))
        })
        .unwrap_or(false)
}

fn script_tag_is_executable(el: &HtmlElement) -> bool {
    let script_type = attr(el, "type")
        .or_else(|| attr(el, "language"))
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    if script_type.is_empty() {
        return true;
    }

    if script_type == "module" {
        return false;
    }

    script_type.contains("javascript")
        || script_type.contains("ecmascript")
        || script_type == "text/jscript"
        || script_type == "application/x-javascript"
}

fn parse_css_rules(css: &str) -> Vec<CssRule> {
    let mut rules = Vec::new();
    let source = strip_css_comments(css);
    let mut declaration_order = 0_usize;
    let mut blocks = Vec::new();
    collect_css_rule_blocks(&source, &mut blocks);

    for (selector_text, dec_text) in blocks {
        if selector_text.is_empty() || dec_text.is_empty() {
            continue;
        }

        let declarations = parse_declaration_entries(&dec_text, &mut declaration_order);
        if declarations.is_empty() {
            continue;
        }
        for part in selector_text.split(',') {
            if let Some(sel) = parse_selector(part.trim()) {
                let specificity = selector_specificity(&sel);
                rules.push(CssRule {
                    sel,
                    specificity,
                    declarations: declarations.clone(),
                });
            }
        }
    }

    rules
}

fn collect_css_rule_blocks(input: &str, out: &mut Vec<(String, String)>) {
    let mut cursor = 0_usize;
    while let Some((selector, body, next_cursor)) = next_css_rule_block(input, cursor) {
        cursor = next_cursor;
        let selector = selector.trim();
        if selector.is_empty() {
            continue;
        }

        if is_css_grouping_at_rule(selector) {
            collect_css_rule_blocks(body, out);
            continue;
        }

        if selector.starts_with('@') {
            continue;
        }

        out.push((selector.to_owned(), body.trim().to_owned()));
    }
}

fn next_css_rule_block(input: &str, from: usize) -> Option<(&str, &str, usize)> {
    let start = skip_css_separators(input, from);
    if start >= input.len() {
        return None;
    }

    let open = find_css_top_level_open_brace(input, start)?;
    let close = find_css_matching_brace(input, open)?;
    Some((&input[start..open], &input[(open + 1)..close], close + 1))
}

fn skip_css_separators(input: &str, mut idx: usize) -> usize {
    while idx < input.len() {
        let byte = input.as_bytes()[idx];
        if byte.is_ascii_whitespace() || byte == b';' {
            idx = idx.saturating_add(1);
            continue;
        }
        break;
    }
    idx
}

fn find_css_top_level_open_brace(input: &str, from: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut idx = from;
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;

    while idx < bytes.len() {
        let byte = bytes[idx];
        if in_single {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }
        if in_double {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'{' if paren_depth == 0 && bracket_depth == 0 => return Some(idx),
            _ => {}
        }

        idx = idx.saturating_add(1);
    }

    None
}

fn find_css_matching_brace(input: &str, open_brace: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.get(open_brace).copied() != Some(b'{') {
        return None;
    }

    let mut idx = open_brace.saturating_add(1);
    let mut depth = 1_u32;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_comment = false;
    let mut escape = false;

    while idx < bytes.len() {
        let byte = bytes[idx];
        let next = bytes.get(idx.saturating_add(1)).copied();

        if in_comment {
            if byte == b'*' && next == Some(b'/') {
                in_comment = false;
                idx = idx.saturating_add(2);
                continue;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_single {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if byte == b'/' && next == Some(b'*') {
            in_comment = true;
            idx = idx.saturating_add(2);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'{' => depth = depth.saturating_add(1),
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }

        idx = idx.saturating_add(1);
    }

    None
}

fn is_css_grouping_at_rule(selector: &str) -> bool {
    let lower = selector.trim().to_ascii_lowercase();
    lower.starts_with("@media")
        || lower.starts_with("@supports")
        || lower.starts_with("@layer")
        || lower.starts_with("@document")
}

fn supports_inherit_keyword(property_name: &str) -> bool {
    matches!(
        property_name,
        "color"
            | "font-size"
            | "text-align"
            | "font-family"
            | "font-weight"
            | "font-style"
            | "text-decoration"
            | "text-decoration-line"
            | "vertical-align"
            | "line-height"
            | "text-transform"
            | "white-space"
            | "list-style"
            | "list-style-type"
    )
}

fn parse_declaration_entries(input: &str, source_order: &mut usize) -> Vec<CssDeclaration> {
    let mut out = Vec::new();

    for chunk in split_css_top_level(input, ';') {
        let trimmed_chunk = chunk.trim();
        let Some(colon_idx) = find_css_top_level_colon(trimmed_chunk) else {
            continue;
        };
        let name_raw = &trimmed_chunk[..colon_idx];
        let value_raw = &trimmed_chunk[(colon_idx + 1)..];

        let name = name_raw.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }

        let (value, important) = split_important(value_raw);
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        let parsed = parse_single_declaration(&name, value);
        let keep_inherit = value.eq_ignore_ascii_case("inherit") && supports_inherit_keyword(&name);
        let keep_raw_property = is_mdn_reference_css_property(&name);
        if parsed.is_empty() && !keep_inherit && !keep_raw_property {
            continue;
        }

        out.push(CssDeclaration {
            name,
            value: value.to_owned(),
            important,
            source_order: *source_order,
            parsed,
        });
        *source_order = source_order.saturating_add(1);
    }

    out
}

fn split_css_top_level(input: &str, delimiter: char) -> Vec<&str> {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut start = 0_usize;
    let mut idx = 0_usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;

    while idx < bytes.len() {
        let byte = bytes[idx];
        if in_single {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }
        if in_double {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {
                if byte == delimiter as u8 && paren_depth == 0 && bracket_depth == 0 {
                    out.push(&input[start..idx]);
                    start = idx.saturating_add(1);
                }
            }
        }

        idx = idx.saturating_add(1);
    }

    if start <= input.len() {
        out.push(&input[start..]);
    }

    out
}

fn find_css_top_level_colon(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut idx = 0_usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;

    while idx < bytes.len() {
        let byte = bytes[idx];
        if in_single {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }
        if in_double {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b':' if paren_depth == 0 && bracket_depth == 0 => return Some(idx),
            _ => {}
        }

        idx = idx.saturating_add(1);
    }

    None
}

fn parse_single_declaration(name: &str, value: &str) -> StyleProps {
    let mut out = StyleProps::default();
    apply_named_declaration(name, value, &mut out);
    out
}

fn strip_css_comments(css: &str) -> String {
    let bytes = css.as_bytes();
    let mut i = 0_usize;
    let mut out = Vec::with_capacity(bytes.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut in_comment = false;
    let mut escape = false;

    while i < bytes.len() {
        let current = bytes[i];
        let next = bytes.get(i.saturating_add(1)).copied();

        if in_comment {
            if current == b'*' && next == Some(b'/') {
                in_comment = false;
                i = i.saturating_add(2);
                continue;
            }
            i = i.saturating_add(1);
            continue;
        }

        if in_single {
            out.push(current);
            if !escape && current == b'\\' {
                escape = true;
            } else if !escape && current == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            i = i.saturating_add(1);
            continue;
        }

        if in_double {
            out.push(current);
            if !escape && current == b'\\' {
                escape = true;
            } else if !escape && current == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            i = i.saturating_add(1);
            continue;
        }

        if current == b'/' && next == Some(b'*') {
            in_comment = true;
            i = i.saturating_add(2);
            continue;
        }

        if current == b'\'' {
            in_single = true;
            out.push(current);
            i = i.saturating_add(1);
            continue;
        }

        if current == b'"' {
            in_double = true;
            out.push(current);
            i = i.saturating_add(1);
            continue;
        }

        out.push(current);
        i = i.saturating_add(1);
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn parse_selector(input: &str) -> Option<Selector> {
    if input.trim().is_empty() {
        return None;
    }

    let bytes = input.trim().as_bytes();
    let mut idx = 0_usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut bracket_depth = 0_u32;
    let mut paren_depth = 0_u32;
    let mut escape = false;
    let mut pending_descendant = false;
    let mut compound = String::new();
    let mut compounds = Vec::new();
    let mut combinators = Vec::new();

    while idx < bytes.len() {
        let byte = bytes[idx];

        if in_single {
            compound.push(byte as char);
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double {
            compound.push(byte as char);
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if bracket_depth > 0 || paren_depth > 0 {
            compound.push(byte as char);
            match byte {
                b'\'' => in_single = true,
                b'"' => in_double = true,
                b'[' => bracket_depth = bracket_depth.saturating_add(1),
                b']' => bracket_depth = bracket_depth.saturating_sub(1),
                b'(' => paren_depth = paren_depth.saturating_add(1),
                b')' => paren_depth = paren_depth.saturating_sub(1),
                _ => {}
            }
            idx = idx.saturating_add(1);
            continue;
        }

        match byte {
            b'\'' => {
                compound.push(byte as char);
                in_single = true;
            }
            b'"' => {
                compound.push(byte as char);
                in_double = true;
            }
            b'[' => {
                compound.push(byte as char);
                bracket_depth = bracket_depth.saturating_add(1);
            }
            b'(' => {
                compound.push(byte as char);
                paren_depth = paren_depth.saturating_add(1);
            }
            b'>' => {
                let trimmed = compound.trim();
                if !trimmed.is_empty() {
                    compounds.push(trimmed.to_owned());
                    compound.clear();
                } else if compounds.is_empty() {
                    return None;
                }

                pending_descendant = false;
                combinators.push(SelectorCombinator::Child);
            }
            b'+' | b'~' => {
                // Adjacent and general sibling selectors are not supported yet.
                return None;
            }
            _ if byte.is_ascii_whitespace() => {
                let trimmed = compound.trim();
                if !trimmed.is_empty() {
                    compounds.push(trimmed.to_owned());
                    compound.clear();
                    pending_descendant = true;
                } else if !compounds.is_empty() && combinators.len() < compounds.len() {
                    pending_descendant = true;
                }
            }
            _ => {
                if pending_descendant {
                    combinators.push(SelectorCombinator::Descendant);
                    pending_descendant = false;
                }
                compound.push(byte as char);
            }
        }

        idx = idx.saturating_add(1);
    }

    let tail = compound.trim();
    if !tail.is_empty() {
        compounds.push(tail.to_owned());
    }

    if compounds.is_empty() || combinators.len() + 1 != compounds.len() {
        return None;
    }

    let mut left_to_right = Vec::with_capacity(compounds.len());
    for raw in compounds {
        let normalized = normalize_selector_compound(&raw);
        let Some(simple) = parse_simple_selector(&normalized) else {
            continue;
        };
        left_to_right.push(simple);
    }

    if left_to_right.is_empty() || left_to_right.len() != combinators.len() + 1 {
        return None;
    }

    let mut segments = Vec::with_capacity(left_to_right.len());
    for index in (0..left_to_right.len()).rev() {
        let combinator_to_next = if index == 0 {
            None
        } else {
            combinators.get(index - 1).copied()
        };
        segments.push(SelectorSegment {
            simple: left_to_right[index].clone(),
            combinator_to_next,
        });
    }

    Some(Selector { segments })
}

fn normalize_selector_compound(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Unsupported selectors should be ignored instead of broadened.
    // Stripping unsupported fragments (e.g. :not(...), [attr]) can
    // incorrectly apply hide/layout rules to unrelated elements.
    if trimmed
        .as_bytes()
        .iter()
        .any(|byte| matches!(*byte, b'[' | b':' | b'\\'))
    {
        return String::new();
    }

    trimmed.to_owned()
}

fn parse_simple_selector(input: &str) -> Option<SimpleSelector> {
    if input.is_empty() || input == "*" {
        return None;
    }

    let mut selector = SimpleSelector::default();
    let bytes = input.as_bytes();
    let mut idx = 0_usize;

    if bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'*')
    {
        let start = idx;
        idx = idx.saturating_add(1);
        while idx < bytes.len() && is_selector_ident_char(bytes[idx]) {
            idx = idx.saturating_add(1);
        }
        let raw_tag = &input[start..idx];
        if raw_tag != "*" {
            selector.tag = Some(raw_tag.to_ascii_lowercase());
        }
    }

    while idx < bytes.len() {
        let marker = bytes[idx];
        if marker != b'#' && marker != b'.' {
            return None;
        }
        idx = idx.saturating_add(1);
        let start = idx;
        while idx < bytes.len() && is_selector_ident_char(bytes[idx]) {
            idx = idx.saturating_add(1);
        }
        if start == idx {
            return None;
        }
        let value = input[start..idx].to_ascii_lowercase();
        if marker == b'#' {
            if selector.id.is_some() {
                return None;
            }
            selector.id = Some(value);
        } else {
            selector.classes.push(value);
        }
    }

    if selector.tag.is_none() && selector.id.is_none() && selector.classes.is_empty() {
        None
    } else {
        Some(selector)
    }
}

fn is_selector_ident_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'
}

fn selector_specificity(selector: &Selector) -> u16 {
    let mut id_count: u16 = 0;
    let mut class_count: u16 = 0;
    let mut tag_count: u16 = 0;

    for segment in &selector.segments {
        if segment.simple.id.is_some() {
            id_count = id_count.saturating_add(1);
        }

        let classes = u16::try_from(segment.simple.classes.len()).unwrap_or(u16::MAX);
        class_count = class_count.saturating_add(classes);

        if segment.simple.tag.is_some() {
            tag_count = tag_count.saturating_add(1);
        }
    }

    id_count
        .saturating_mul(100)
        .saturating_add(class_count.saturating_mul(10))
        .saturating_add(tag_count)
}

fn matches_selector(sel: &Selector, el: &HtmlElement, ancestors: &[SelectorSubject]) -> bool {
    if sel.segments.is_empty() {
        return false;
    }

    if !matches_simple_selector_element(&sel.segments[0].simple, el) {
        return false;
    }

    let mut ancestor_limit = ancestors.len();
    for segment_index in 0..sel.segments.len().saturating_sub(1) {
        let combinator = sel.segments[segment_index]
            .combinator_to_next
            .unwrap_or(SelectorCombinator::Descendant);
        let next_simple = &sel.segments[segment_index + 1].simple;

        match combinator {
            SelectorCombinator::Child => {
                if ancestor_limit == 0 {
                    return false;
                }
                let parent_index = ancestor_limit.saturating_sub(1);
                let parent = &ancestors[parent_index];
                if !matches_simple_selector_subject(next_simple, parent) {
                    return false;
                }
                ancestor_limit = parent_index;
            }
            SelectorCombinator::Descendant => {
                let mut found: Option<usize> = None;
                let mut search = ancestor_limit;
                while search > 0 {
                    search = search.saturating_sub(1);
                    if matches_simple_selector_subject(next_simple, &ancestors[search]) {
                        found = Some(search);
                        break;
                    }
                }
                let Some(found_index) = found else {
                    return false;
                };
                ancestor_limit = found_index;
            }
        }
    }

    true
}

fn matches_simple_selector_element(simple: &SimpleSelector, el: &HtmlElement) -> bool {
    if let Some(tag) = &simple.tag {
        let expected = canonical_element_tag(tag);
        let actual = canonical_element_tag(el.tag.as_str());
        if actual != expected {
            return false;
        }
    }

    if let Some(id) = &simple.id {
        let element_id = attr(el, "id");
        if !element_id
            .map(|value| value.eq_ignore_ascii_case(id))
            .unwrap_or(false)
        {
            return false;
        }
    }

    if !simple.classes.is_empty() {
        let classes = attr(el, "class");
        for class_name in &simple.classes {
            let found = classes
                .map(|value| {
                    value
                        .split_ascii_whitespace()
                        .any(|candidate| candidate.eq_ignore_ascii_case(class_name))
                })
                .unwrap_or(false);
            if !found {
                return false;
            }
        }
    }

    true
}

fn matches_simple_selector_subject(simple: &SimpleSelector, subject: &SelectorSubject) -> bool {
    if let Some(tag) = &simple.tag {
        let expected = canonical_element_tag(tag);
        let actual = canonical_element_tag(subject.tag.as_str());
        if actual != expected {
            return false;
        }
    }

    if let Some(id) = &simple.id {
        if !subject
            .id
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(id))
        {
            return false;
        }
    }

    if !simple.classes.is_empty() {
        for class_name in &simple.classes {
            if !subject
                .classes
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(class_name))
            {
                return false;
            }
        }
    }

    true
}

fn selector_subject(el: &HtmlElement) -> SelectorSubject {
    let id = attr(el, "id").map(ToOwned::to_owned);
    let classes = attr(el, "class")
        .map(|value| {
            value
                .split_ascii_whitespace()
                .map(|part| part.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    SelectorSubject {
        tag: el.tag.clone(),
        id,
        classes,
    }
}

#[cfg(test)]
fn parse_declarations(input: &str) -> StyleProps {
    let mut out = StyleProps::default();

    for chunk in input.split(';') {
        let Some((name_raw, value_raw)) = chunk.split_once(':') else {
            continue;
        };

        let name = name_raw.trim().to_ascii_lowercase();
        let (value, _) = split_important(value_raw);
        let value = value.trim();
        if name.is_empty() || value.is_empty() {
            continue;
        }

        apply_named_declaration(&name, value, &mut out);
        if is_mdn_reference_css_property(&name) {
            out.raw_css.insert(name, value.to_owned());
        }
    }

    apply_raw_css_aliases(&mut out);
    out
}

fn apply_named_declaration(name: &str, value: &str, out: &mut StyleProps) {
    match name {
        "display" => {
            if let Some(v) = parse_display(value) {
                out.display = Some(v);
            }
        }
        "visibility" => {
            if let Some(v) = parse_visibility_hidden(value) {
                out.visibility_hidden = Some(v);
            }
        }
        "opacity" => {
            if let Some(v) = parse_opacity(value) {
                out.opacity = Some(v);
            }
        }
        "text-align" => {
            if let Some(v) = parse_text_align(value) {
                out.text_align = Some(v);
            }
        }
        "color" => {
            if let Some(v) = parse_color(value) {
                out.color = Some(v);
            }
        }
        "background" | "background-color" => {
            if let Some(v) = parse_color(value).or_else(|| parse_first_color_in_css_value(value)) {
                out.bg = Some(v);
            }
        }
        "font-size" => {
            if let Some(v) = parse_length(value) {
                out.font_size = Some(v.max(6.0));
            }
        }
        "font-family" => {
            if let Some(v) = parse_font_family(value) {
                out.font_family = Some(v);
            }
        }
        "font" => {
            apply_font_shorthand(value, out);
        }
        "line-height" => {
            if let Some(v) = parse_line_height(value) {
                out.line_height = Some(v.max(0.0));
            }
        }
        "flex-direction" => {
            if let Some(v) = parse_flex_direction(value) {
                out.flex_direction = Some(v);
            }
        }
        "flex-wrap" => {
            if let Some(v) = parse_flex_wrap(value) {
                out.flex_wrap = Some(v);
            }
        }
        "justify-content" => {
            if let Some(v) = parse_justify_content(value) {
                out.justify_content = Some(v);
            }
        }
        "align-items" => {
            if let Some(v) = parse_align_items(value) {
                out.align_items = Some(v);
            }
        }
        "align-content" => {
            if let Some(v) = parse_align_content(value) {
                out.align_content = Some(v);
            }
        }
        "place-content" => apply_place_content_shorthand(value, out),
        "place-items" => apply_place_items_shorthand(value, out),
        "gap" | "row-gap" | "column-gap" => {
            if let Some(v) = parse_length(value) {
                out.gap = Some(v.max(0.0));
            }
        }
        "font-weight" => {
            if let Some(v) = parse_font_weight(value) {
                out.bold = Some(v);
            }
        }
        "font-style" => {
            if let Some(v) = parse_font_style(value) {
                out.italic = Some(v);
            }
        }
        "text-decoration" | "text-decoration-line" => {
            apply_text_decoration(value, out);
        }
        "list-style-type" => {
            if let Some(v) = parse_list_style_type(value) {
                out.list_style_type = Some(v.to_owned());
            }
        }
        "list-style" => {
            if let Some(v) = parse_list_style_type(value) {
                out.list_style_type = Some(v.to_owned());
            }
        }
        "position" => {
            if let Some(v) = parse_position_mode(value) {
                out.position = Some(v);
            }
        }
        "top" => {
            if let Some(v) = parse_length(value) {
                out.inset_top = Some(v);
            }
        }
        "right" => {
            if let Some(v) = parse_length(value) {
                out.inset_right = Some(v);
            }
        }
        "bottom" => {
            if let Some(v) = parse_length(value) {
                out.inset_bottom = Some(v);
            }
        }
        "left" => {
            if let Some(v) = parse_length(value) {
                out.inset_left = Some(v);
            }
        }
        "z-index" => {
            if let Some(v) = parse_z_index(value) {
                out.z_index = Some(v);
            }
        }
        "overflow" => apply_overflow_shorthand(value, out),
        "overflow-x" => {
            if let Some(v) = parse_overflow_mode(value) {
                out.overflow_x = Some(v);
            }
        }
        "overflow-y" => {
            if let Some(v) = parse_overflow_mode(value) {
                out.overflow_y = Some(v);
            }
        }
        "vertical-align" => {
            if let Some(v) = parse_vertical_align(value) {
                out.script = Some(v);
            }
        }
        "width" => {
            if let Some(v) = parse_length(value) {
                out.width = Some(v.max(1.0));
            } else if let Some(percent) = parse_percentage(value) {
                out.width_percent = Some(percent);
            }
        }
        "height" => {
            if let Some(v) = parse_length(value) {
                out.height = Some(v.max(1.0));
            }
        }
        "min-height" => {
            if let Some(v) = parse_length(value) {
                out.min_height = Some(v.max(0.0));
            }
        }
        "max-height" => {
            if let Some(v) = parse_length(value) {
                out.max_height = Some(v.max(0.0));
            }
        }
        "min-width" => {
            if let Some(v) = parse_length(value) {
                out.min_width = Some(v.max(0.0));
            }
        }
        "max-width" => {
            if let Some(v) = parse_length(value) {
                out.max_width = Some(v.max(0.0));
            }
        }
        "margin" => {
            if let Some(v) = parse_margin_edges(value) {
                out.margin.apply(&v.non_negative());
            }
        }
        "margin-top" => set_margin_edge_value(&mut out.margin.top, value),
        "margin-right" => set_margin_edge_value(&mut out.margin.right, value),
        "margin-bottom" => set_margin_edge_value(&mut out.margin.bottom, value),
        "margin-left" => set_margin_edge_value(&mut out.margin.left, value),
        "padding" => {
            if let Some(v) = parse_edges(value) {
                out.padding.apply(&v.non_negative());
            }
        }
        "padding-top" => set_edge_value(&mut out.padding.top, value),
        "padding-right" => set_edge_value(&mut out.padding.right, value),
        "padding-bottom" => set_edge_value(&mut out.padding.bottom, value),
        "padding-left" => set_edge_value(&mut out.padding.left, value),
        "border" => apply_border_shorthand(value, out),
        "border-width" => {
            if let Some(v) = parse_edges(value) {
                out.border_width.apply(&v.non_negative());
            }
        }
        "border-color" => {
            if let Some(v) = parse_color(value) {
                out.border_color = Some(v);
            }
        }
        "border-top" => apply_border_side_shorthand(value, out, EdgeSide::Top),
        "border-right" => apply_border_side_shorthand(value, out, EdgeSide::Right),
        "border-bottom" => apply_border_side_shorthand(value, out, EdgeSide::Bottom),
        "border-left" => apply_border_side_shorthand(value, out, EdgeSide::Left),
        "border-top-width" => set_edge_value(&mut out.border_width.top, value),
        "border-right-width" => set_edge_value(&mut out.border_width.right, value),
        "border-bottom-width" => set_edge_value(&mut out.border_width.bottom, value),
        "border-left-width" => set_edge_value(&mut out.border_width.left, value),
        _ => {}
    }
}

fn apply_raw_css_aliases(style: &mut StyleProps) {
    let raw_css = style.raw_css.clone();
    let raw = |name: &str| raw_css_value(&raw_css, name).map(ToOwned::to_owned);

    if style.text_transform.is_none()
        && let Some(value) = raw("text-transform").as_deref().and_then(parse_text_transform)
    {
        style.text_transform = Some(value);
    }

    if style.white_space.is_none()
        && let Some(value) = raw("white-space").as_deref().and_then(parse_white_space_mode)
    {
        style.white_space = Some(value);
    }

    if style.list_style_type.is_none() {
        if let Some(value) = raw("list-style-type")
            .as_deref()
            .and_then(parse_list_style_type)
        {
            style.list_style_type = Some(value.to_owned());
        } else if let Some(value) = raw("list-style")
            .as_deref()
            .and_then(parse_list_style_type)
        {
            style.list_style_type = Some(value.to_owned());
        }
    }

    if style.border_radius.is_none()
        && let Some(value) = raw("border-radius").as_deref().and_then(parse_border_radius)
    {
        style.border_radius = Some(value);
    }

    if style.width.is_none() && style.width_percent.is_none() {
        if let Some(value) = raw("inline-size") {
            apply_width_like_value(style, value.as_str());
        }
    }
    if style.height.is_none()
        && let Some(value) = raw("block-size")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.height = Some(value.max(1.0));
    }

    if style.min_width.is_none()
        && let Some(value) = raw("min-inline-size")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.min_width = Some(value);
    }
    if style.max_width.is_none()
        && let Some(value) = raw("max-inline-size")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.max_width = Some(value);
    }

    if style.min_height.is_none() {
        if let Some(value) = raw("min-height")
            .or_else(|| raw("min-block-size"))
            .as_deref()
            .and_then(parse_non_negative_length)
        {
            style.min_height = Some(value);
        }
    }
    if style.max_height.is_none() {
        if let Some(value) = raw("max-height")
            .or_else(|| raw("max-block-size"))
            .as_deref()
            .and_then(parse_non_negative_length)
        {
            style.max_height = Some(value);
        }
    }

    if style.gap.is_none() {
        let gap_value = raw("gap")
            .or_else(|| raw("row-gap"))
            .or_else(|| raw("column-gap"))
            .as_deref()
            .and_then(parse_non_negative_length_from_first_token);
        if let Some(value) = gap_value {
            style.gap = Some(value);
        }
    }

    if style.flex_wrap.is_none()
        && let Some(value) = raw("flex-wrap").as_deref().and_then(parse_flex_wrap)
    {
        style.flex_wrap = Some(value);
    }

    if let Some(value) = raw("margin-inline").as_deref().and_then(parse_margin_pair) {
        if style.margin.left.is_none() {
            style.margin.left = Some(value.0);
        }
        if style.margin.right.is_none() {
            style.margin.right = Some(value.1);
        }
    }
    if let Some(value) = raw("margin-block").as_deref().and_then(parse_margin_pair) {
        if style.margin.top.is_none() {
            style.margin.top = Some(value.0);
        }
        if style.margin.bottom.is_none() {
            style.margin.bottom = Some(value.1);
        }
    }
    if style.margin.left.is_none()
        && let Some(value) = raw("margin-inline-start")
            .as_deref()
            .and_then(parse_margin_length)
    {
        style.margin.left = Some(value);
    }
    if style.margin.right.is_none()
        && let Some(value) = raw("margin-inline-end")
            .as_deref()
            .and_then(parse_margin_length)
    {
        style.margin.right = Some(value);
    }
    if style.margin.top.is_none()
        && let Some(value) = raw("margin-block-start")
            .as_deref()
            .and_then(parse_margin_length)
    {
        style.margin.top = Some(value);
    }
    if style.margin.bottom.is_none()
        && let Some(value) = raw("margin-block-end")
            .as_deref()
            .and_then(parse_margin_length)
    {
        style.margin.bottom = Some(value);
    }

    if let Some(value) = raw("padding-inline").as_deref().and_then(parse_length_pair) {
        if style.padding.left.is_none() {
            style.padding.left = Some(value.0);
        }
        if style.padding.right.is_none() {
            style.padding.right = Some(value.1);
        }
    }
    if let Some(value) = raw("padding-block").as_deref().and_then(parse_length_pair) {
        if style.padding.top.is_none() {
            style.padding.top = Some(value.0);
        }
        if style.padding.bottom.is_none() {
            style.padding.bottom = Some(value.1);
        }
    }
    if style.padding.left.is_none()
        && let Some(value) = raw("padding-inline-start")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.padding.left = Some(value);
    }
    if style.padding.right.is_none()
        && let Some(value) = raw("padding-inline-end")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.padding.right = Some(value);
    }
    if style.padding.top.is_none()
        && let Some(value) = raw("padding-block-start")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.padding.top = Some(value);
    }
    if style.padding.bottom.is_none()
        && let Some(value) = raw("padding-block-end")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.padding.bottom = Some(value);
    }

    if let Some(value) = raw("border-inline-width").as_deref().and_then(parse_length_pair) {
        if style.border_width.left.is_none() {
            style.border_width.left = Some(value.0);
        }
        if style.border_width.right.is_none() {
            style.border_width.right = Some(value.1);
        }
    }
    if let Some(value) = raw("border-block-width").as_deref().and_then(parse_length_pair) {
        if style.border_width.top.is_none() {
            style.border_width.top = Some(value.0);
        }
        if style.border_width.bottom.is_none() {
            style.border_width.bottom = Some(value.1);
        }
    }
    if style.border_width.left.is_none()
        && let Some(value) = raw("border-inline-start-width")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.border_width.left = Some(value);
    }
    if style.border_width.right.is_none()
        && let Some(value) = raw("border-inline-end-width")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.border_width.right = Some(value);
    }
    if style.border_width.top.is_none()
        && let Some(value) = raw("border-block-start-width")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.border_width.top = Some(value);
    }
    if style.border_width.bottom.is_none()
        && let Some(value) = raw("border-block-end-width")
            .as_deref()
            .and_then(parse_non_negative_length)
    {
        style.border_width.bottom = Some(value);
    }

    if style.align_items.is_none() {
        if let Some(value) = raw("place-items")
            .as_deref()
            .and_then(parse_first_token)
            .and_then(parse_align_items)
        {
            style.align_items = Some(value);
        } else if let Some(value) = raw("align-items")
            .as_deref()
            .and_then(parse_align_items)
        {
            style.align_items = Some(value);
        }
    }

    if style.align_content.is_none()
        && let Some(value) = raw("align-content")
            .as_deref()
            .and_then(parse_align_content)
    {
        style.align_content = Some(value);
    }

    if let Some(value) = raw("place-content") {
        apply_place_content_shorthand(value.as_str(), style);
    }

    if style.position.is_none()
        && let Some(value) = raw("position").as_deref().and_then(parse_position_mode)
    {
        style.position = Some(value);
    }
    if style.z_index.is_none()
        && let Some(value) = raw("z-index").as_deref().and_then(parse_z_index)
    {
        style.z_index = Some(value);
    }

    if style.inset_top.is_none()
        && let Some(value) = raw("top").as_deref().and_then(parse_length)
    {
        style.inset_top = Some(value);
    }
    if style.inset_right.is_none()
        && let Some(value) = raw("right").as_deref().and_then(parse_length)
    {
        style.inset_right = Some(value);
    }
    if style.inset_bottom.is_none()
        && let Some(value) = raw("bottom").as_deref().and_then(parse_length)
    {
        style.inset_bottom = Some(value);
    }
    if style.inset_left.is_none()
        && let Some(value) = raw("left").as_deref().and_then(parse_length)
    {
        style.inset_left = Some(value);
    }

    if let Some(value) = raw("inset").as_deref().and_then(parse_edges) {
        if style.inset_top.is_none() {
            style.inset_top = value.top;
        }
        if style.inset_right.is_none() {
            style.inset_right = value.right;
        }
        if style.inset_bottom.is_none() {
            style.inset_bottom = value.bottom;
        }
        if style.inset_left.is_none() {
            style.inset_left = value.left;
        }
    }

    if let Some(value) = raw("inset-inline").as_deref().and_then(parse_length_pair) {
        if style.inset_left.is_none() {
            style.inset_left = Some(value.0);
        }
        if style.inset_right.is_none() {
            style.inset_right = Some(value.1);
        }
    }
    if let Some(value) = raw("inset-block").as_deref().and_then(parse_length_pair) {
        if style.inset_top.is_none() {
            style.inset_top = Some(value.0);
        }
        if style.inset_bottom.is_none() {
            style.inset_bottom = Some(value.1);
        }
    }

    if style.overflow_x.is_none() || style.overflow_y.is_none() {
        if let Some(value) = raw("overflow") {
            let mut parsed = StyleProps::default();
            apply_overflow_shorthand(value.as_str(), &mut parsed);
            if style.overflow_x.is_none() {
                style.overflow_x = parsed.overflow_x;
            }
            if style.overflow_y.is_none() {
                style.overflow_y = parsed.overflow_y;
            }
        }
    }
    if style.overflow_x.is_none()
        && let Some(value) = raw("overflow-x").as_deref().and_then(parse_overflow_mode)
    {
        style.overflow_x = Some(value);
    }
    if style.overflow_y.is_none()
        && let Some(value) = raw("overflow-y").as_deref().and_then(parse_overflow_mode)
    {
        style.overflow_y = Some(value);
    }
    if style.overflow_x.is_none()
        && let Some(value) = raw("overflow-inline")
            .as_deref()
            .and_then(parse_overflow_mode)
    {
        style.overflow_x = Some(value);
    }
    if style.overflow_y.is_none()
        && let Some(value) = raw("overflow-block")
            .as_deref()
            .and_then(parse_overflow_mode)
    {
        style.overflow_y = Some(value);
    }
}

fn raw_css_value<'a>(raw_css: &'a HashMap<String, String>, name: &str) -> Option<&'a str> {
    raw_css
        .get(name)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn apply_width_like_value(style: &mut StyleProps, value: &str) {
    if let Some(length) = parse_non_negative_length(value) {
        style.width = Some(length.max(1.0));
    } else if let Some(percent) = parse_percentage(value) {
        style.width_percent = Some(percent);
    }
}

fn parse_non_negative_length(value: &str) -> Option<f32> {
    parse_length(value).map(|value| value.max(0.0))
}

fn parse_non_negative_length_from_first_token(value: &str) -> Option<f32> {
    parse_first_token(value).and_then(parse_non_negative_length)
}

fn parse_length_pair(value: &str) -> Option<(f32, f32)> {
    parse_pair_with(value, parse_non_negative_length)
}

fn parse_margin_pair(value: &str) -> Option<(f32, f32)> {
    parse_pair_with(value, parse_margin_length)
}

fn parse_pair_with(value: &str, parser: fn(&str) -> Option<f32>) -> Option<(f32, f32)> {
    let mut parts = value.split_ascii_whitespace();
    let first = parts.next().and_then(parser)?;
    let second = parts.next().and_then(parser).unwrap_or(first);
    if parts.next().is_some() {
        return None;
    }
    Some((first, second))
}

fn parse_first_token(value: &str) -> Option<&str> {
    value
        .split_ascii_whitespace()
        .next()
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

fn parse_list_style_type(value: &str) -> Option<&'static str> {
    for token in value.split_ascii_whitespace() {
        let lower = token.trim().to_ascii_lowercase();
        let normalized = match lower.as_str() {
            "none" => "none",
            "disc" => "disc",
            "circle" => "circle",
            "square" => "square",
            "decimal" | "decimal-leading-zero" => "decimal",
            "lower-alpha" | "lower-latin" | "a" => "lower-alpha",
            "upper-alpha" | "upper-latin" | "A" => "upper-alpha",
            "lower-roman" | "i" => "lower-roman",
            "upper-roman" | "I" => "upper-roman",
            _ => continue,
        };
        return Some(normalized);
    }
    None
}

fn parse_first_color_in_css_value(value: &str) -> Option<egui::Color32> {
    for raw in value.split(|ch: char| ch.is_whitespace() || ch == ',' || ch == '(' || ch == ')') {
        let token = raw.trim();
        if token.is_empty() {
            continue;
        }
        if let Some(color) = parse_color(token) {
            return Some(color);
        }
    }
    None
}

fn parse_border_radius(value: &str) -> Option<f32> {
    let first_group = value.split('/').next()?.trim();
    let first_token = parse_first_token(first_group)?;
    parse_non_negative_length(first_token)
}

fn parse_text_transform(value: &str) -> Option<TextTransform> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(TextTransform::None),
        "uppercase" => Some(TextTransform::Uppercase),
        "lowercase" => Some(TextTransform::Lowercase),
        "capitalize" => Some(TextTransform::Capitalize),
        _ => None,
    }
}

fn parse_white_space_mode(value: &str) -> Option<WhiteSpaceMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(WhiteSpaceMode::Normal),
        "nowrap" => Some(WhiteSpaceMode::NoWrap),
        "pre" => Some(WhiteSpaceMode::Pre),
        "pre-wrap" => Some(WhiteSpaceMode::PreWrap),
        "pre-line" => Some(WhiteSpaceMode::PreLine),
        _ => None,
    }
}

fn split_important(value: &str) -> (&str, bool) {
    let trimmed = value.trim();
    if trimmed.len() < "!important".len() {
        return (trimmed, false);
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.ends_with("!important") {
        let cutoff = trimmed.len().saturating_sub("!important".len());
        (trimmed[..cutoff].trim_end(), true)
    } else {
        (trimmed, false)
    }
}

fn parse_display(value: &str) -> Option<Display> {
    match value.trim().to_ascii_lowercase().as_str() {
        "block" | "table" | "list-item" => Some(Display::Block),
        "flex" | "inline-flex" => Some(Display::Flex),
        "grid" | "inline-grid" => Some(Display::Grid),
        "inline" | "inline-block" => Some(Display::Inline),
        "none" => Some(Display::None),
        _ => None,
    }
}

fn parse_visibility_hidden(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "visible" => Some(false),
        "hidden" | "collapse" => Some(true),
        _ => None,
    }
}

fn parse_opacity(value: &str) -> Option<f32> {
    let raw = value.trim();
    if raw.eq_ignore_ascii_case("initial") {
        return Some(1.0);
    }
    if let Some(percent) = parse_percentage(raw) {
        return Some((percent / 100.0).clamp(0.0, 1.0));
    }
    let parsed = raw.parse::<f32>().ok()?;
    Some(parsed.clamp(0.0, 1.0))
}

fn parse_flex_direction(value: &str) -> Option<FlexDirection> {
    match value.trim().to_ascii_lowercase().as_str() {
        "row" | "row-reverse" => Some(FlexDirection::Row),
        "column" | "column-reverse" => Some(FlexDirection::Column),
        _ => None,
    }
}

fn parse_flex_wrap(value: &str) -> Option<FlexWrap> {
    match value.trim().to_ascii_lowercase().as_str() {
        "nowrap" => Some(FlexWrap::NoWrap),
        "wrap" | "wrap-reverse" => Some(FlexWrap::Wrap),
        _ => None,
    }
}

fn parse_justify_content(value: &str) -> Option<JustifyContent> {
    match value.trim().to_ascii_lowercase().as_str() {
        "flex-start" | "start" | "left" => Some(JustifyContent::Start),
        "center" => Some(JustifyContent::Center),
        "flex-end" | "end" | "right" => Some(JustifyContent::End),
        "space-between" => Some(JustifyContent::SpaceBetween),
        "space-around" => Some(JustifyContent::SpaceAround),
        "space-evenly" => Some(JustifyContent::SpaceEvenly),
        _ => None,
    }
}

fn parse_align_items(value: &str) -> Option<AlignItems> {
    match value.trim().to_ascii_lowercase().as_str() {
        "flex-start" | "start" | "left" | "top" => Some(AlignItems::Start),
        "center" => Some(AlignItems::Center),
        "flex-end" | "end" | "right" | "bottom" => Some(AlignItems::End),
        "stretch" => Some(AlignItems::Stretch),
        _ => None,
    }
}

fn parse_align_content(value: &str) -> Option<AlignContent> {
    match value.trim().to_ascii_lowercase().as_str() {
        "flex-start" | "start" | "top" => Some(AlignContent::Start),
        "center" => Some(AlignContent::Center),
        "flex-end" | "end" | "bottom" => Some(AlignContent::End),
        "space-between" => Some(AlignContent::SpaceBetween),
        "space-around" => Some(AlignContent::SpaceAround),
        "space-evenly" => Some(AlignContent::SpaceEvenly),
        "stretch" => Some(AlignContent::Stretch),
        _ => None,
    }
}

fn parse_position_mode(value: &str) -> Option<PositionMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "static" => Some(PositionMode::Static),
        "relative" => Some(PositionMode::Relative),
        "absolute" => Some(PositionMode::Absolute),
        "fixed" => Some(PositionMode::Fixed),
        "sticky" => Some(PositionMode::Sticky),
        _ => None,
    }
}

fn parse_overflow_mode(value: &str) -> Option<OverflowMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "visible" => Some(OverflowMode::Visible),
        "hidden" | "clip" => Some(OverflowMode::Hidden),
        "auto" => Some(OverflowMode::Auto),
        "scroll" => Some(OverflowMode::Scroll),
        _ => None,
    }
}

fn parse_z_index(value: &str) -> Option<i32> {
    let raw = value.trim();
    if raw.eq_ignore_ascii_case("auto") {
        return Some(0);
    }
    raw.parse::<i32>().ok()
}

fn parse_text_align(value: &str) -> Option<TextAlign> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" | "start" => Some(TextAlign::Left),
        "center" => Some(TextAlign::Center),
        "right" | "end" => Some(TextAlign::Right),
        "justify" => Some(TextAlign::Justify),
        _ => None,
    }
}

fn parse_font_family(value: &str) -> Option<FontFamilyChoice> {
    let lower = value.to_ascii_lowercase();
    if lower.contains("monospace")
        || lower.contains("menlo")
        || lower.contains("consolas")
        || lower.contains("courier")
    {
        return Some(FontFamilyChoice::Monospace);
    }

    if lower.contains("sans-serif")
        || lower.contains("serif")
        || lower.contains("system-ui")
        || lower.contains("ui-sans-serif")
        || lower.contains("ui-serif")
        || lower.contains("arial")
        || lower.contains("helvetica")
    {
        return Some(FontFamilyChoice::Proportional);
    }

    None
}

fn parse_font_weight(value: &str) -> Option<bool> {
    let raw = value.trim().to_ascii_lowercase();
    match raw.as_str() {
        "normal" | "lighter" | "thin" | "extra-light" | "ultra-light" | "light" => Some(false),
        "bold" | "bolder" | "semibold" | "demibold" | "extra-bold" | "ultra-bold" | "black"
        | "heavy" | "medium" => Some(true),
        _ => {
            let numeric = raw.parse::<u16>().ok()?;
            Some(numeric >= 500)
        }
    }
}

fn parse_font_style(value: &str) -> Option<bool> {
    let raw = value.trim().to_ascii_lowercase();
    if raw == "normal" {
        Some(false)
    } else if raw == "italic" || raw.starts_with("oblique") {
        Some(true)
    } else {
        None
    }
}

fn apply_font_shorthand(value: &str, out: &mut StyleProps) {
    let mut parsed_size = None;
    let mut parsed_line_height = None;
    let mut family_parts = Vec::new();

    for token in value.split_ascii_whitespace() {
        if parsed_size.is_none() {
            if let Some(v) = parse_font_style(token) {
                out.italic = Some(v);
                continue;
            }
            if let Some(v) = parse_font_weight(token) {
                out.bold = Some(v);
                continue;
            }
            if let Some((size, line_height)) = parse_font_size_token(token) {
                parsed_size = Some(size.max(6.0));
                parsed_line_height = line_height.map(|v| v.max(0.0));
                continue;
            }
            if token.eq_ignore_ascii_case("normal")
                || token.eq_ignore_ascii_case("small-caps")
                || token.eq_ignore_ascii_case("caption")
                || token.eq_ignore_ascii_case("menu")
                || token.eq_ignore_ascii_case("icon")
                || token.eq_ignore_ascii_case("message-box")
                || token.eq_ignore_ascii_case("status-bar")
            {
                continue;
            }
        }

        if parsed_size.is_some() {
            family_parts.push(token);
        }
    }

    if let Some(size) = parsed_size {
        out.font_size = Some(size);
    }
    if let Some(line_height) = parsed_line_height {
        out.line_height = Some(line_height);
    }
    if !family_parts.is_empty() {
        let family_raw = family_parts.join(" ");
        if let Some(v) = parse_font_family(&family_raw) {
            out.font_family = Some(v);
        }
    }
}

fn parse_font_size_token(token: &str) -> Option<(f32, Option<f32>)> {
    if let Some((size_raw, line_raw)) = token.split_once('/') {
        let size = parse_length(size_raw.trim())?;
        let line_height = parse_line_height(line_raw.trim());
        return Some((size, line_height));
    }

    parse_length(token).map(|size| (size, None))
}

fn apply_text_decoration(value: &str, out: &mut StyleProps) {
    let raw = value.trim().to_ascii_lowercase();
    if raw.contains("none") {
        out.underline = Some(false);
        out.strike = Some(false);
        return;
    }
    if raw.contains("underline") {
        out.underline = Some(true);
    }
    if raw.contains("line-through") {
        out.strike = Some(true);
    }
}

fn apply_overflow_shorthand(value: &str, out: &mut StyleProps) {
    let mut parts = value
        .split_ascii_whitespace()
        .filter(|part| !part.trim().is_empty());
    let Some(first_raw) = parts.next() else {
        return;
    };
    let first = parse_overflow_mode(first_raw);
    let second = parts.next().and_then(parse_overflow_mode).or(first);

    if let Some(x) = first {
        out.overflow_x = Some(x);
    }
    if let Some(y) = second {
        out.overflow_y = Some(y);
    }
}

fn apply_place_items_shorthand(value: &str, out: &mut StyleProps) {
    let mut parts = value
        .split_ascii_whitespace()
        .map(str::trim)
        .filter(|part| !part.is_empty());
    let Some(first) = parts.next() else {
        return;
    };
    let second = parts.next().unwrap_or(first);
    if let Some(v) = parse_align_items(first) {
        out.align_items = Some(v);
    }
    if out.justify_content.is_none() && let Some(v) = parse_justify_content(second) {
        out.justify_content = Some(v);
    }
}

fn apply_place_content_shorthand(value: &str, out: &mut StyleProps) {
    let mut parts = value
        .split_ascii_whitespace()
        .map(str::trim)
        .filter(|part| !part.is_empty());
    let Some(first) = parts.next() else {
        return;
    };
    let second = parts.next().unwrap_or(first);
    if let Some(v) = parse_align_content(first) {
        out.align_content = Some(v);
    }
    if let Some(v) = parse_justify_content(second) {
        out.justify_content = Some(v);
    }
}

fn parse_vertical_align(value: &str) -> Option<ScriptPosition> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Some(ScriptPosition::Baseline),
        "sub" => Some(ScriptPosition::Sub),
        "super" | "sup" => Some(ScriptPosition::Sup),
        _ => None,
    }
}

fn set_edge_value(target: &mut Option<f32>, value: &str) {
    if let Some(parsed) = parse_length(value) {
        *target = Some(parsed.max(0.0));
    }
}

fn set_margin_edge_value(target: &mut Option<f32>, value: &str) {
    if let Some(parsed) = parse_margin_length(value) {
        *target = Some(if parsed.is_infinite() {
            parsed
        } else {
            parsed.max(0.0)
        });
    }
}

fn parse_edges(value: &str) -> Option<Edges> {
    let values = value
        .split_ascii_whitespace()
        .map(parse_length)
        .collect::<Option<Vec<_>>>()?;

    match values.as_slice() {
        [all] => Some(Edges::all(*all)),
        [vertical, horizontal] => Some(Edges {
            top: Some(*vertical),
            right: Some(*horizontal),
            bottom: Some(*vertical),
            left: Some(*horizontal),
        }),
        [top, horizontal, bottom] => Some(Edges {
            top: Some(*top),
            right: Some(*horizontal),
            bottom: Some(*bottom),
            left: Some(*horizontal),
        }),
        [top, right, bottom, left] => Some(Edges {
            top: Some(*top),
            right: Some(*right),
            bottom: Some(*bottom),
            left: Some(*left),
        }),
        _ => None,
    }
}

fn parse_margin_edges(value: &str) -> Option<Edges> {
    let values = value
        .split_ascii_whitespace()
        .map(parse_margin_length)
        .collect::<Option<Vec<_>>>()?;

    match values.as_slice() {
        [all] => Some(Edges::all(*all)),
        [vertical, horizontal] => Some(Edges {
            top: Some(*vertical),
            right: Some(*horizontal),
            bottom: Some(*vertical),
            left: Some(*horizontal),
        }),
        [top, horizontal, bottom] => Some(Edges {
            top: Some(*top),
            right: Some(*horizontal),
            bottom: Some(*bottom),
            left: Some(*horizontal),
        }),
        [top, right, bottom, left] => Some(Edges {
            top: Some(*top),
            right: Some(*right),
            bottom: Some(*bottom),
            left: Some(*left),
        }),
        _ => None,
    }
}

fn parse_margin_length(value: &str) -> Option<f32> {
    let raw = value.trim();
    if raw.eq_ignore_ascii_case("auto") {
        return Some(f32::INFINITY);
    }
    parse_length(raw)
}

fn parse_line_height(value: &str) -> Option<f32> {
    let raw = value.trim();
    if raw.eq_ignore_ascii_case("normal") {
        return None;
    }
    if let Some(px) = raw.strip_suffix("px") {
        return px.trim().parse::<f32>().ok();
    }
    if let Some(percent) = raw.strip_suffix('%') {
        let parsed = percent.trim().parse::<f32>().ok()?;
        return Some(parsed / 100.0);
    }
    if let Some(em) = raw.strip_suffix("em") {
        return em.trim().parse::<f32>().ok();
    }
    if let Some(rem) = raw.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok();
    }
    raw.parse::<f32>().ok()
}

fn apply_border_shorthand(value: &str, style: &mut StyleProps) {
    let mut width: Option<f32> = None;
    let mut color: Option<egui::Color32> = None;
    for token in value.split_ascii_whitespace() {
        if width.is_none() {
            width = parse_length(token);
        }
        if color.is_none() {
            color = parse_color(token);
        }
    }

    if let Some(border_width) = width {
        style.border_width = Edges::all(border_width.max(0.0));
    }
    if let Some(border_color) = color {
        style.border_color = Some(border_color);
    }
}

fn apply_border_side_shorthand(value: &str, style: &mut StyleProps, side: EdgeSide) {
    let mut width: Option<f32> = None;
    let mut color: Option<egui::Color32> = None;
    for token in value.split_ascii_whitespace() {
        if width.is_none() {
            width = parse_length(token);
        }
        if color.is_none() {
            color = parse_color(token);
        }
    }

    if let Some(v) = width {
        match side {
            EdgeSide::Top => style.border_width.top = Some(v.max(0.0)),
            EdgeSide::Right => style.border_width.right = Some(v.max(0.0)),
            EdgeSide::Bottom => style.border_width.bottom = Some(v.max(0.0)),
            EdgeSide::Left => style.border_width.left = Some(v.max(0.0)),
        }
    }

    if let Some(v) = color {
        style.border_color = Some(v);
    }
}

fn parse_length(value: &str) -> Option<f32> {
    let raw = value.trim();
    if raw.is_empty() || raw.eq_ignore_ascii_case("auto") || raw.ends_with('%') {
        return None;
    }

    if let Some(px) = raw.strip_suffix("px") {
        return px.trim().parse::<f32>().ok();
    }
    if let Some(rem) = raw.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|v| v * 16.0);
    }
    if let Some(em) = raw.strip_suffix("em") {
        return em.trim().parse::<f32>().ok().map(|v| v * 16.0);
    }
    if let Some(pt) = raw.strip_suffix("pt") {
        return pt.trim().parse::<f32>().ok().map(|v| v * (96.0 / 72.0));
    }

    raw.parse::<f32>().ok()
}

fn parse_percentage(value: &str) -> Option<f32> {
    let raw = value.trim();
    let percent = raw.strip_suffix('%')?.trim();
    let parsed = percent.parse::<f32>().ok()?;
    Some(parsed.clamp(0.0, 1000.0))
}

fn parse_color(value: &str) -> Option<egui::Color32> {
    let raw = value.trim().to_ascii_lowercase();

    if let Some(hex) = raw.strip_prefix('#') {
        return parse_hex(hex);
    }

    if raw.starts_with("rgb(") && raw.ends_with(')') {
        return parse_rgb_function(&raw, false);
    }

    if raw.starts_with("rgba(") && raw.ends_with(')') {
        return parse_rgb_function(&raw, true);
    }

    match raw.as_str() {
        "black" => Some(egui::Color32::BLACK),
        "white" => Some(egui::Color32::WHITE),
        "gray" | "grey" => Some(egui::Color32::GRAY),
        "red" => Some(egui::Color32::RED),
        "green" => Some(egui::Color32::GREEN),
        "blue" => Some(egui::Color32::BLUE),
        "yellow" => Some(egui::Color32::YELLOW),
        "transparent" => Some(egui::Color32::TRANSPARENT),
        _ => None,
    }
}

fn parse_hex(v: &str) -> Option<egui::Color32> {
    if v.len() == 3 {
        let r = u8::from_str_radix(&v[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&v[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&v[2..3].repeat(2), 16).ok()?;
        return Some(egui::Color32::from_rgb(r, g, b));
    }

    if v.len() == 4 {
        let r = u8::from_str_radix(&v[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&v[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&v[2..3].repeat(2), 16).ok()?;
        let a = u8::from_str_radix(&v[3..4].repeat(2), 16).ok()?;
        return Some(egui::Color32::from_rgba_premultiplied(r, g, b, a));
    }

    if v.len() == 6 {
        let r = u8::from_str_radix(&v[0..2], 16).ok()?;
        let g = u8::from_str_radix(&v[2..4], 16).ok()?;
        let b = u8::from_str_radix(&v[4..6], 16).ok()?;
        return Some(egui::Color32::from_rgb(r, g, b));
    }

    if v.len() == 8 {
        let r = u8::from_str_radix(&v[0..2], 16).ok()?;
        let g = u8::from_str_radix(&v[2..4], 16).ok()?;
        let b = u8::from_str_radix(&v[4..6], 16).ok()?;
        let a = u8::from_str_radix(&v[6..8], 16).ok()?;
        return Some(egui::Color32::from_rgba_premultiplied(r, g, b, a));
    }

    None
}

fn parse_rgb_function(v: &str, with_alpha: bool) -> Option<egui::Color32> {
    let inside = if with_alpha {
        v.strip_prefix("rgba(")?.strip_suffix(')')?
    } else {
        v.strip_prefix("rgb(")?.strip_suffix(')')?
    };
    let parts = inside.split(',').map(str::trim).collect::<Vec<_>>();
    if (with_alpha && parts.len() != 4) || (!with_alpha && parts.len() != 3) {
        return None;
    }

    let r = parse_rgb_channel(parts[0])?;
    let g = parse_rgb_channel(parts[1])?;
    let b = parse_rgb_channel(parts[2])?;
    let a = if with_alpha {
        parse_alpha_channel(parts[3])?
    } else {
        255
    };
    Some(egui::Color32::from_rgba_premultiplied(r, g, b, a))
}

fn parse_rgb_channel(value: &str) -> Option<u8> {
    let raw = value.trim();
    if let Some(percent) = raw.strip_suffix('%') {
        let value = percent.trim().parse::<f32>().ok()?.clamp(0.0, 100.0);
        return Some(((value / 100.0) * 255.0).round() as u8);
    }
    let value = raw.parse::<f32>().ok()?.clamp(0.0, 255.0);
    Some(value.round() as u8)
}

fn parse_alpha_channel(value: &str) -> Option<u8> {
    let raw = value.trim();
    if let Some(percent) = raw.strip_suffix('%') {
        let value = percent.trim().parse::<f32>().ok()?.clamp(0.0, 100.0);
        return Some(((value / 100.0) * 255.0).round() as u8);
    }
    let value = raw.parse::<f32>().ok()?.clamp(0.0, 1.0);
    Some((value * 255.0).round() as u8)
}

fn form_control_state_key(base_url: &str, el: &HtmlElement, kind: &str) -> String {
    let element_ptr = el as *const HtmlElement as usize;
    let id = attr(el, "id").unwrap_or("");
    let name = attr(el, "name").unwrap_or("");
    format!("{base_url}|{kind}|{element_ptr:x}|{id}|{name}")
}

fn form_runtime_key(el: &HtmlElement) -> String {
    let element_ptr = el as *const HtmlElement as usize;
    let id = attr(el, "id").unwrap_or("");
    format!("{element_ptr:x}|{id}")
}

fn set_active_form_field(ctx: &mut Ctx<'_>, name: &str, value: Option<String>) {
    let Some(form) = ctx.form_stack.last() else {
        return;
    };
    let key = form.key.clone();
    let field_name = name.trim();
    if field_name.is_empty() {
        return;
    }
    let fields = ctx.form_fields.entry(key).or_default();
    match value {
        Some(value) => {
            fields.insert(field_name.to_owned(), value);
        }
        None => {
            fields.remove(field_name);
        }
    }
}

fn emit_inline_event(ctx: &mut Ctx<'_>, kind: DomEventKind, el: &HtmlElement, attr_name: &str) {
    let Some(handler) = attr(el, attr_name).map(str::trim).map(ToOwned::to_owned) else {
        return;
    };
    if handler.is_empty() {
        return;
    }
    ctx.action.dom_events.push(DomEventRequest {
        kind,
        target_id: attr(el, "id").map(ToOwned::to_owned),
        inline_handler: handler,
    });
}

fn submit_active_form(
    ctx: &mut Ctx<'_>,
    submit_name: Option<String>,
    submit_value: Option<String>,
    trigger: Option<&HtmlElement>,
) {
    let Some(form) = ctx.form_stack.last().cloned() else {
        return;
    };

    if let Some(handler) = form.onsubmit.clone() {
        let trimmed = handler.trim();
        if !trimmed.is_empty() {
            ctx.action.dom_events.push(DomEventRequest {
                kind: DomEventKind::Submit,
                target_id: form.form_id.clone(),
                inline_handler: trimmed.to_owned(),
            });
        }
    }

    if !form.method.eq_ignore_ascii_case("get") {
        return;
    }

    let mut fields = ctx
        .form_fields
        .get(&form.key)
        .cloned()
        .unwrap_or_else(HashMap::new);

    if let Some(name) = submit_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        fields.insert(name.to_owned(), submit_value.unwrap_or_default());
    }

    if let Some(url) = build_form_submit_url(&form.action_url, &fields) {
        ctx.action.navigate_to = Some(url);
    } else if let Some(trigger) = trigger
        && let Some(formaction) =
            attr(trigger, "formaction").and_then(|value| resolve_link(ctx.base_url, value))
    {
        ctx.action.navigate_to = Some(formaction);
    }
}

fn build_form_submit_url(action_url: &str, fields: &HashMap<String, String>) -> Option<String> {
    let mut parsed = Url::parse(action_url).ok()?;
    let mut pairs = fields
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect::<Vec<_>>();
    pairs.sort_by(|left, right| left.0.cmp(right.0));

    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (name, value) in pairs {
        serializer.append_pair(name, value);
    }
    let query = serializer.finish();
    if query.is_empty() {
        parsed.set_query(None);
    } else {
        parsed.set_query(Some(&query));
    }
    Some(parsed.to_string())
}

fn attr<'a>(el: &'a HtmlElement, name: &str) -> Option<&'a str> {
    el.attrs
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

fn has_attr(el: &HtmlElement, name: &str) -> bool {
    attr(el, name).is_some()
}

fn parse_usize_attr(el: &HtmlElement, name: &str) -> Option<usize> {
    attr(el, name)
        .and_then(|raw| raw.trim().parse::<isize>().ok())
        .and_then(|value| usize::try_from(value).ok())
}

fn parse_background_resource_attr<'a>(el: &'a HtmlElement) -> Option<&'a str> {
    attr(el, "background").filter(|value| !value.trim().is_empty())
}

fn truncate_text_to_max_chars(text: &mut String, max_chars: Option<usize>) {
    let Some(max_chars) = max_chars.filter(|value| *value > 0) else {
        return;
    };

    if text.chars().count() <= max_chars {
        return;
    }

    let truncated = text.chars().take(max_chars).collect::<String>();
    text.clear();
    text.push_str(&truncated);
}

fn resolve_link(base_url: &str, href: &str) -> Option<String> {
    if href.trim().is_empty() {
        return None;
    }

    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(href.to_owned());
    }

    let base = Url::parse(base_url).ok()?;
    let joined = base.join(href).ok()?;
    match joined.scheme() {
        "http" | "https" => Some(joined.to_string()),
        _ => None,
    }
}
fn tokenize(source: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if starts_with(bytes, i, b"<!--") {
            i = skip_comment(bytes, i);
            continue;
        }

        if bytes[i] == b'<' {
            if starts_with(bytes, i, b"</") {
                if let Some((tok, next)) = parse_end_tag(bytes, i) {
                    out.push(tok);
                    i = next;
                    continue;
                }
            } else if starts_with(bytes, i, b"<!") {
                i = skip_decl(bytes, i);
                continue;
            } else if let Some((tok, next)) = parse_start_tag(bytes, i) {
                let mut raw_text_tag: Option<String> = None;
                if let Token::Start {
                    name, self_closing, ..
                } = &tok
                {
                    if !*self_closing && is_raw_text_tag(name) {
                        raw_text_tag = Some(name.clone());
                    }
                }

                out.push(tok);
                i = next;

                if let Some(tag_name) = raw_text_tag {
                    let (raw_text, closing_end) = parse_raw_text_until_end_tag(bytes, i, &tag_name);
                    if !raw_text.is_empty() {
                        out.push(Token::Text(raw_text));
                    }

                    if let Some(closing_end) = closing_end {
                        out.push(Token::End { name: tag_name });
                        i = closing_end;
                    } else {
                        i = bytes.len();
                    }
                }

                continue;
            }
        }

        let (txt, next) = parse_text(bytes, i);
        if !txt.is_empty() {
            out.push(Token::Text(txt));
        }
        i = next;
    }

    out
}

fn build_tree(tokens: Vec<Token>) -> HtmlElement {
    let mut stack = vec![HtmlElement {
        tag: "document".to_owned(),
        attrs: Vec::new(),
        children: Vec::new(),
    }];

    for token in tokens {
        match token {
            Token::Text(text) => {
                if let Some(cur) = stack.last_mut() {
                    cur.children.push(HtmlNode::Text(decode_entities(&text)));
                }
            }
            Token::Start {
                name,
                attrs,
                self_closing,
            } => {
                let el = HtmlElement {
                    tag: name.clone(),
                    attrs,
                    children: Vec::new(),
                };

                if self_closing || is_void(&name) {
                    if let Some(cur) = stack.last_mut() {
                        cur.children.push(HtmlNode::Element(el));
                    }
                } else {
                    stack.push(el);
                }
            }
            Token::End { name } => {
                while stack.len() > 1 {
                    let node = match stack.pop() {
                        Some(v) => v,
                        None => break,
                    };
                    let matched = node.tag == name;
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(HtmlNode::Element(node));
                    }
                    if matched {
                        break;
                    }
                }
            }
        }
    }

    while stack.len() > 1 {
        let node = match stack.pop() {
            Some(v) => v,
            None => break,
        };
        if let Some(parent) = stack.last_mut() {
            parent.children.push(HtmlNode::Element(node));
        }
    }

    stack.pop().unwrap_or(HtmlElement {
        tag: "document".to_owned(),
        attrs: Vec::new(),
        children: Vec::new(),
    })
}

fn find_title(root: &HtmlElement) -> Option<String> {
    find_title_nodes(&root.children)
}

fn find_title_nodes(nodes: &[HtmlNode]) -> Option<String> {
    for node in nodes {
        match node {
            HtmlNode::Text(_) => {}
            HtmlNode::Element(el) => {
                if el.tag == "title" {
                    let t = collapse_whitespace(&collect_text(&el.children));
                    if !t.is_empty() {
                        return Some(t);
                    }
                }
                if let Some(found) = find_title_nodes(&el.children) {
                    return Some(found);
                }
            }
        }
    }
    None
}

fn collect_text(nodes: &[HtmlNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            HtmlNode::Text(t) => out.push_str(t),
            HtmlNode::Element(el) => {
                if is_non_rendered_element_tag(el.tag.as_str()) {
                    continue;
                }
                out.push_str(&collect_text(&el.children));
            }
        }
    }
    out
}

fn collect_renderable_text(
    nodes: &[HtmlNode],
    sheet: &StyleSheet,
    inherited: &StyleProps,
    ancestors: &mut Vec<SelectorSubject>,
    out: &mut String,
) {
    for node in nodes {
        match node {
            HtmlNode::Text(text) => {
                if !text.trim().is_empty() {
                    out.push(' ');
                    out.push_str(text);
                }
            }
            HtmlNode::Element(el) => {
                if is_non_rendered_element_tag(el.tag.as_str()) {
                    continue;
                }

                let style = style_for(el, sheet, inherited, ancestors);
                if matches!(style.display, Some(Display::None)) {
                    continue;
                }

                ancestors.push(selector_subject(el));
                collect_renderable_text(&el.children, sheet, &style, ancestors, out);
                ancestors.pop();
            }
        }
    }
}

fn collect_static_fallback_text(nodes: &[HtmlNode], out: &mut String) {
    for node in nodes {
        match node {
            HtmlNode::Text(text) => {
                if !text.trim().is_empty() {
                    out.push(' ');
                    out.push_str(text);
                }
            }
            HtmlNode::Element(el) => {
                if is_static_fallback_ignored_tag(el.tag.as_str()) {
                    continue;
                }
                collect_static_fallback_text(&el.children, out);
            }
        }
    }
}

fn find_first_element<'a>(nodes: &'a [HtmlNode], tag: &str) -> Option<&'a HtmlElement> {
    for node in nodes {
        let HtmlNode::Element(el) = node else {
            continue;
        };

        if el.tag.eq_ignore_ascii_case(tag) {
            return Some(el);
        }

        if let Some(found) = find_first_element(&el.children, tag) {
            return Some(found);
        }
    }

    None
}

fn collapse_whitespace(input: &str) -> String {
    let mut out = String::new();
    let mut ws = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !ws {
                out.push(' ');
                ws = true;
            }
        } else {
            out.push(ch);
            ws = false;
        }
    }
    out.trim().to_owned()
}

fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0_usize;

    while let Some(rel_amp) = input[cursor..].find('&') {
        let amp = cursor + rel_amp;
        out.push_str(&input[cursor..amp]);

        let rest = &input[(amp + 1)..];
        let Some(rel_semi) = rest.find(';') else {
            out.push('&');
            cursor = amp + 1;
            continue;
        };

        let semi = amp + 1 + rel_semi;
        let entity = &input[(amp + 1)..semi];
        if let Some(decoded) = decode_entity(entity) {
            out.push_str(&decoded);
            cursor = semi + 1;
        } else {
            out.push('&');
            cursor = amp + 1;
        }
    }

    out.push_str(&input[cursor..]);
    out
}

fn decode_entity(entity: &str) -> Option<String> {
    match entity {
        "nbsp" => Some(" ".to_owned()),
        "amp" => Some("&".to_owned()),
        "lt" => Some("<".to_owned()),
        "gt" => Some(">".to_owned()),
        "quot" => Some("\"".to_owned()),
        "apos" => Some("'".to_owned()),
        _ => {
            if let Some(hex) = entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
            {
                let value = u32::from_str_radix(hex, 16).ok()?;
                char::from_u32(value).map(|ch| ch.to_string())
            } else if let Some(dec) = entity.strip_prefix('#') {
                let value = dec.parse::<u32>().ok()?;
                char::from_u32(value).map(|ch| ch.to_string())
            } else {
                None
            }
        }
    }
}

fn starts_with(bytes: &[u8], i: usize, pat: &[u8]) -> bool {
    let end = i.saturating_add(pat.len());
    end <= bytes.len() && &bytes[i..end] == pat
}

fn skip_comment(bytes: &[u8], start: usize) -> usize {
    let mut i = start.saturating_add(4);
    while i + 2 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>' {
            return i + 3;
        }
        i += 1;
    }
    bytes.len()
}

fn skip_decl(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 2;
    while i < bytes.len() {
        if bytes[i] == b'>' {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

fn parse_text(bytes: &[u8], start: usize) -> (String, usize) {
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'<' {
        i += 1;
    }
    (String::from_utf8_lossy(&bytes[start..i]).to_string(), i)
}

fn parse_raw_text_until_end_tag(
    bytes: &[u8],
    start: usize,
    tag_name: &str,
) -> (String, Option<usize>) {
    let tag_bytes = tag_name.as_bytes();
    let mut i = start;

    while i < bytes.len() {
        if bytes[i] != b'<' || i + 2 + tag_bytes.len() > bytes.len() {
            i = i.saturating_add(1);
            continue;
        }
        if bytes[i + 1] != b'/' {
            i = i.saturating_add(1);
            continue;
        }

        let name_start = i + 2;
        let name_end = name_start + tag_bytes.len();
        if !bytes_eq_ignore_ascii_case(&bytes[name_start..name_end], tag_bytes) {
            i = i.saturating_add(1);
            continue;
        }

        let mut close = name_end;
        while close < bytes.len() && bytes[close].is_ascii_whitespace() {
            close = close.saturating_add(1);
        }

        if close < bytes.len() && bytes[close] == b'>' {
            let text = String::from_utf8_lossy(&bytes[start..i]).to_string();
            return (text, Some(close + 1));
        }

        i = i.saturating_add(1);
    }

    (String::from_utf8_lossy(&bytes[start..]).to_string(), None)
}

fn bytes_eq_ignore_ascii_case(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(lhs, rhs)| lhs.eq_ignore_ascii_case(rhs))
}

fn parse_end_tag(bytes: &[u8], start: usize) -> Option<(Token, usize)> {
    let mut i = start + 2;
    skip_spaces(bytes, &mut i);
    let begin = i;
    while i < bytes.len() && is_name_char(bytes[i]) {
        i += 1;
    }
    if i == begin {
        return None;
    }

    let name = String::from_utf8_lossy(&bytes[begin..i]).to_ascii_lowercase();
    while i < bytes.len() && bytes[i] != b'>' {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }

    Some((Token::End { name }, i + 1))
}

fn parse_start_tag(bytes: &[u8], start: usize) -> Option<(Token, usize)> {
    let mut i = start + 1;
    skip_spaces(bytes, &mut i);
    let begin = i;
    while i < bytes.len() && is_name_char(bytes[i]) {
        i += 1;
    }
    if i == begin {
        return None;
    }

    let name = String::from_utf8_lossy(&bytes[begin..i]).to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut self_closing = false;

    loop {
        skip_spaces(bytes, &mut i);
        if i >= bytes.len() {
            return None;
        }

        if bytes[i] == b'>' {
            i += 1;
            break;
        }

        if bytes[i] == b'/' {
            self_closing = true;
            i += 1;
            skip_spaces(bytes, &mut i);
            if i < bytes.len() && bytes[i] == b'>' {
                i += 1;
                break;
            }
            continue;
        }

        let a_start = i;
        while i < bytes.len() && is_name_char(bytes[i]) {
            i += 1;
        }
        if i == a_start {
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            break;
        }

        let a_name = String::from_utf8_lossy(&bytes[a_start..i]).to_ascii_lowercase();
        skip_spaces(bytes, &mut i);

        let mut val = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            skip_spaces(bytes, &mut i);
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let v_start = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                val = String::from_utf8_lossy(&bytes[v_start..i]).to_string();
                if i < bytes.len() && bytes[i] == q {
                    i += 1;
                }
            } else {
                let v_start = i;
                while i < bytes.len()
                    && !bytes[i].is_ascii_whitespace()
                    && bytes[i] != b'>'
                    && bytes[i] != b'/'
                {
                    i += 1;
                }
                val = String::from_utf8_lossy(&bytes[v_start..i]).to_string();
            }
        }

        attrs.push((a_name, decode_entities(&val)));
    }

    Some((
        Token::Start {
            name,
            attrs,
            self_closing,
        },
        i,
    ))
}

fn skip_spaces(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

fn is_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':')
}

fn is_raw_text_tag(tag: &str) -> bool {
    matches!(
        canonical_element_tag(tag),
        "plaintext" | "script" | "style" | "textarea" | "title" | "xmp"
    )
}

fn is_void(tag: &str) -> bool {
    let tag = canonical_element_tag(tag);
    matches!(
        tag,
        "area"
            | "base"
            | "basefont"
            | "bgsound"
            | "br"
            | "col"
            | "embed"
            | "frame"
            | "hr"
            | "img"
            | "input"
            | "keygen"
            | "link"
            | "menuitem"
            | "meta"
            | "nextid"
            | "param"
            | "source"
            | "spacer"
            | "track"
            | "wbr"
    )
}

fn is_block(tag: &str) -> bool {
    let tag = canonical_element_tag(tag);
    matches!(
        tag,
        "html"
            | "body"
            | "main"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "nav"
            | "aside"
            | "address"
            | "search"
            | "div"
            | "p"
            | "pre"
            | "plaintext"
            | "xmp"
            | "hr"
            | "form"
            | "fieldset"
            | "legend"
            | "dialog"
            | "ul"
            | "ol"
            | "menu"
            | "dir"
            | "li"
            | "dl"
            | "dt"
            | "dd"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "hgroup"
            | "figure"
            | "figcaption"
            | "details"
            | "summary"
            | "center"
            | "marquee"
            | "table"
            | "caption"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "td"
            | "th"
            | "frameset"
            | "noframes"
            | "multicol"
            | "blockquote"
            | "video"
            | "audio"
            | "canvas"
            | "svg"
            | "math"
            | "iframe"
            | "fencedframe"
            | "portal"
            | "frame"
            | "embed"
            | "object"
            | "applet"
            | "noscript"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AlignContent, AlignItems, Display, FlexDirection, FlexWrap, FontFamilyChoice, HtmlDocument,
        HtmlElement, HtmlNode, JustifyContent, MDN_REFERENCE_ATTRIBUTES, MDN_REFERENCE_ELEMENTS,
        OverflowMode, PositionMode, ScriptDescriptor, ScriptPosition, StyleProps, StyleSheet,
        TextAlign, TextEffects, TextTransform, WhiteSpaceMode, collapse_whitespace, decode_entities, find_first_element,
        is_likely_screen_reader_only, is_mdn_reference_attribute, is_mdn_reference_css_property,
        is_mdn_reference_element, is_void, mdn_reference_css_properties,
        normalize_text_for_render, ordered_list_marker, parse_color, parse_css_rules,
        parse_declarations, parse_legacy_font_size, resolve_link, selector_subject, style_for,
        unordered_list_marker,
    };
    use eframe::egui::Color32;

    #[test]
    fn parses_title() {
        let src =
            "<html><head><title>Hello</title></head><body><h1>Hi</h1><p>World</p></body></html>";
        let doc = HtmlDocument::parse(src);
        assert_eq!(doc.title.as_deref(), Some("Hello"));
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn mdn_reference_registry_is_wired() {
        for tag in MDN_REFERENCE_ELEMENTS {
            assert!(
                is_mdn_reference_element(tag),
                "missing MDN element tag: {tag}"
            );
        }
    }

    #[test]
    fn mdn_attribute_registry_is_wired() {
        for attr in MDN_REFERENCE_ATTRIBUTES {
            assert!(
                is_mdn_reference_attribute(attr),
                "missing MDN attribute: {attr}"
            );
        }
    }

    #[test]
    fn mdn_css_property_registry_is_wired() {
        let properties = mdn_reference_css_properties();
        assert!(
            properties.len() >= 500,
            "expected large MDN CSS property list, got {}",
            properties.len()
        );
        for property in properties {
            assert!(
                is_mdn_reference_css_property(property),
                "missing MDN CSS property: {property}"
            );
        }
    }

    #[test]
    fn recognizes_wildcard_and_event_handler_attributes() {
        assert!(is_mdn_reference_attribute("data-user-id"));
        assert!(is_mdn_reference_attribute("aria-label"));
        assert!(is_mdn_reference_attribute("onclick"));
        assert!(!is_mdn_reference_attribute("made-up-attr"));
    }

    #[test]
    fn parses_all_mdn_reference_attributes_without_crashing() {
        for attr in MDN_REFERENCE_ATTRIBUTES {
            let attr_name = if *attr == "data-*" {
                "data-probe"
            } else {
                attr
            };
            let html = format!("<html><body><div {attr_name}=\"x\"></div></body></html>");
            let doc = HtmlDocument::parse(&html);
            let div = find_first_element(&doc.root.children, "div")
                .unwrap_or_else(|| panic!("missing div while parsing attr {attr_name}"));
            assert!(
                div.attrs.iter().any(|(name, _)| name == attr_name),
                "attribute {attr_name} missing from parsed DOM"
            );
        }
    }

    #[test]
    fn parses_all_mdn_reference_elements_without_crashing() {
        for tag in MDN_REFERENCE_ELEMENTS {
            if matches!(*tag, "html" | "head" | "body" | "title") {
                continue;
            }

            let snippet = if is_void(tag) {
                format!("<{tag}>")
            } else {
                format!("<{tag}>ok</{tag}>")
            };
            let html = format!("<html><body>{snippet}</body></html>");
            let doc = HtmlDocument::parse(&html);
            assert!(
                !doc.root.children.is_empty(),
                "parser failed to create DOM for tag <{tag}>"
            );
        }
    }

    #[test]
    fn builds_static_text_fallback_from_noscript() {
        let src =
            "<html><body><noscript>Enable JS <a href=\"/retry\">here</a></noscript></body></html>";
        let doc = HtmlDocument::parse(src);
        assert_eq!(doc.visible_text_len(), 0);
        let fallback = doc.static_text_fallback(120);
        assert!(fallback.contains("Enable JS here"));
    }

    #[test]
    fn renderable_text_excludes_display_none_content() {
        let src = "<html><body><div style=\"display:none\">Hidden</div><p>Shown</p></body></html>";
        let doc = HtmlDocument::parse(src);
        assert_eq!(doc.renderable_text_len(), "Shown".len());
    }

    #[test]
    fn renderable_text_ignores_noscript_content() {
        let src = "<html><body><noscript>Enable JS</noscript></body></html>";
        let doc = HtmlDocument::parse(src);
        assert_eq!(doc.renderable_text_len(), 0);
    }

    #[test]
    fn renderable_text_ignores_template_content() {
        let src = "<html><body><template><p>Hidden</p></template><p>Shown</p></body></html>";
        let doc = HtmlDocument::parse(src);
        assert_eq!(doc.renderable_text_len(), "Shown".len());
    }

    #[test]
    fn ignores_noscript_styles_in_stylesheet_extraction() {
        let src = "<html><head><style>p{color:red}</style></head>\
                   <body><noscript><style>p{display:none}</style></noscript><p>ok</p></body></html>";
        let doc = HtmlDocument::parse(src);
        assert_eq!(doc.inline_style_tag_count(), 1);
        assert_eq!(doc.css_rule_count(), 1);
    }

    #[test]
    fn detects_screen_reader_only_styles() {
        let style = parse_declarations(
            "width:1px;height:1px;margin:-1px;padding:0;border:0;line-height:1px;",
        );
        assert!(is_likely_screen_reader_only(&style));
    }

    #[test]
    fn does_not_flag_regular_visible_styles() {
        let style = parse_declarations("width:120px;height:24px;margin:0;padding:0;border:0;");
        assert!(!is_likely_screen_reader_only(&style));
    }

    #[test]
    fn resolves_relative_links() {
        let resolved = resolve_link("https://example.com/docs/index.html", "../about");
        assert_eq!(resolved.as_deref(), Some("https://example.com/about"));
    }

    #[test]
    fn collapses_ws() {
        assert_eq!(collapse_whitespace("hello   \n world"), "hello world");
    }

    #[test]
    fn parses_css() {
        let rules = parse_css_rules("p{color:#ff0000}.big{font-size:24px}#hero{display:none}");
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].specificity, 1);
        assert_eq!(rules[1].specificity, 10);
        assert_eq!(rules[2].specificity, 100);
    }

    #[test]
    fn parses_nested_media_css_rules() {
        let css = "@media screen and (min-width: 100px){ .hero{display:block} #q{width:100%} }";
        let rules = parse_css_rules(css);
        assert_eq!(rules.len(), 2);
        assert!(rules.iter().any(|rule| {
            rule.sel.segments.iter().any(|segment| {
                segment
                    .simple
                    .classes
                    .iter()
                    .any(|class_name| class_name == "hero")
            })
        }));
        assert!(rules.iter().any(|rule| {
            rule.sel
                .segments
                .iter()
                .any(|segment| segment.simple.id.as_deref() == Some("q"))
        }));
    }

    #[test]
    fn keeps_semicolons_inside_css_function_values() {
        let css = ".icon{background-image:url(\"data:image/svg+xml;utf8,<svg></svg>\");color:#111}";
        let rules = parse_css_rules(css);
        assert_eq!(rules.len(), 1);
        assert!(
            rules[0]
                .declarations
                .iter()
                .any(|declaration| declaration.name == "color")
        );
    }

    #[test]
    fn keeps_unimplemented_mdn_properties_in_rule_declarations() {
        let css = ".pane{scrollbar-gutter:stable both-edges;color:#111}";
        let rules = parse_css_rules(css);
        assert_eq!(rules.len(), 1);
        assert!(
            rules[0]
                .declarations
                .iter()
                .any(|declaration| declaration.name == "scrollbar-gutter")
        );
    }

    #[test]
    fn drops_unknown_non_mdn_properties() {
        let css = ".pane{made-up-css-prop:123;color:#111}";
        let rules = parse_css_rules(css);
        assert_eq!(rules.len(), 1);
        assert!(
            rules[0]
                .declarations
                .iter()
                .all(|declaration| declaration.name != "made-up-css-prop")
        );
    }

    #[test]
    fn parses_css_phase1_text_style() {
        let style = parse_declarations(
            "text-align:center;font-weight:700;font-style:italic;\
             text-decoration:underline line-through;vertical-align:super",
        );
        assert_eq!(style.text_align, Some(TextAlign::Center));
        assert_eq!(style.bold, Some(true));
        assert_eq!(style.italic, Some(true));
        assert_eq!(style.underline, Some(true));
        assert_eq!(style.strike, Some(true));
        assert_eq!(style.script, Some(ScriptPosition::Sup));
    }

    #[test]
    fn parses_font_family_declaration() {
        let style = parse_declarations("font-family: 'Courier New', monospace;");
        assert_eq!(style.font_family, Some(FontFamilyChoice::Monospace));
    }

    #[test]
    fn parses_font_shorthand_declaration() {
        let style = parse_declarations("font: italic 700 18px/24px Arial, sans-serif;");
        assert_eq!(style.italic, Some(true));
        assert_eq!(style.bold, Some(true));
        assert_eq!(style.font_size, Some(18.0));
        assert_eq!(style.line_height, Some(24.0));
        assert_eq!(style.font_family, Some(FontFamilyChoice::Proportional));
    }

    #[test]
    fn parses_flex_declarations() {
        let style = parse_declarations(
            "display:flex;flex-direction:row;justify-content:center;align-items:stretch;gap:12px",
        );
        assert_eq!(style.display, Some(Display::Flex));
        assert_eq!(style.flex_direction, Some(FlexDirection::Row));
        assert_eq!(style.justify_content, Some(JustifyContent::Center));
        assert_eq!(style.align_items, Some(AlignItems::Stretch));
        assert_eq!(style.gap, Some(12.0));
    }

    #[test]
    fn parses_flex_wrap_and_align_content_declarations() {
        let style = parse_declarations(
            "display:flex;flex-wrap:wrap;align-content:space-between;place-content:center end;",
        );
        assert_eq!(style.display, Some(Display::Flex));
        assert_eq!(style.flex_wrap, Some(FlexWrap::Wrap));
        assert_eq!(style.align_content, Some(AlignContent::Center));
        assert_eq!(style.justify_content, Some(JustifyContent::End));
    }

    #[test]
    fn parses_position_overflow_and_z_index_declarations() {
        let style = parse_declarations(
            "position:absolute;top:10px;right:16px;z-index:42;overflow:hidden auto;",
        );
        assert_eq!(style.position, Some(PositionMode::Absolute));
        assert_eq!(style.inset_top, Some(10.0));
        assert_eq!(style.inset_right, Some(16.0));
        assert_eq!(style.z_index, Some(42));
        assert_eq!(style.overflow_x, Some(OverflowMode::Hidden));
        assert_eq!(style.overflow_y, Some(OverflowMode::Auto));
    }

    #[test]
    fn parses_visibility_and_opacity_declarations() {
        let hidden = parse_declarations("visibility:hidden;opacity:0;");
        assert_eq!(hidden.visibility_hidden, Some(true));
        assert_eq!(hidden.opacity, Some(0.0));

        let visible = parse_declarations("visibility:visible;opacity:1.5;");
        assert_eq!(visible.visibility_hidden, Some(false));
        assert_eq!(visible.opacity, Some(1.0));

        let percent = parse_declarations("opacity:25%;");
        assert_eq!(percent.opacity, Some(0.25));
    }

    #[test]
    fn parses_width_percent_and_constraints() {
        let style = parse_declarations("width:100%;min-width:54px;max-width:496px;");
        assert_eq!(style.width_percent, Some(100.0));
        assert_eq!(style.min_width, Some(54.0));
        assert_eq!(style.max_width, Some(496.0));
    }

    #[test]
    fn parses_box_model_declarations() {
        let style = parse_declarations(
            "margin: 10px 20px 30px 40px; padding: 5px 8px;\
             border: 2px solid #102030; line-height: 18px;",
        );
        assert_eq!(style.margin.top, Some(10.0));
        assert_eq!(style.margin.right, Some(20.0));
        assert_eq!(style.margin.bottom, Some(30.0));
        assert_eq!(style.margin.left, Some(40.0));
        assert_eq!(style.padding.top, Some(5.0));
        assert_eq!(style.padding.right, Some(8.0));
        assert_eq!(style.padding.bottom, Some(5.0));
        assert_eq!(style.padding.left, Some(8.0));
        assert_eq!(style.border_width.top, Some(2.0));
        assert_eq!(style.border_color, Some(Color32::from_rgb(16, 32, 48)));
        assert_eq!(style.line_height, Some(18.0));
    }

    #[test]
    fn parses_logical_box_model_and_size_aliases() {
        let style = parse_declarations(
            "inline-size:50%;block-size:120px;\
             min-block-size:40px;max-block-size:200px;\
             margin-inline:10px 20px;padding-block:4px 8px;\
             border-inline-width:3px 6px;",
        );
        assert_eq!(style.width_percent, Some(50.0));
        assert_eq!(style.height, Some(120.0));
        assert_eq!(style.min_height, Some(40.0));
        assert_eq!(style.max_height, Some(200.0));
        assert_eq!(style.margin.left, Some(10.0));
        assert_eq!(style.margin.right, Some(20.0));
        assert_eq!(style.padding.top, Some(4.0));
        assert_eq!(style.padding.bottom, Some(8.0));
        assert_eq!(style.border_width.left, Some(3.0));
        assert_eq!(style.border_width.right, Some(6.0));
    }

    #[test]
    fn parses_text_transform_whitespace_and_radius_aliases() {
        let style = parse_declarations("text-transform:uppercase;white-space:pre-line;border-radius:12px;");
        assert_eq!(style.text_transform, Some(TextTransform::Uppercase));
        assert_eq!(style.white_space, Some(WhiteSpaceMode::PreLine));
        assert_eq!(style.border_radius, Some(12.0));
    }

    #[test]
    fn normalizes_text_with_white_space_and_transform() {
        let style = StyleProps {
            text_transform: Some(TextTransform::Uppercase),
            white_space: Some(WhiteSpaceMode::PreLine),
            ..StyleProps::default()
        };
        let rendered = normalize_text_for_render("hello   world\nnext\tline", &style, &TextEffects::default());
        assert_eq!(rendered, "HELLO WORLD\nNEXT LINE");
    }

    #[test]
    fn parses_margin_auto_and_rgba_colors() {
        let style = parse_declarations(
            "margin: 0 auto; background: #0000; color: rgba(255, 255, 255, 0.5);",
        );
        assert_eq!(style.margin.top, Some(0.0));
        assert_eq!(style.margin.bottom, Some(0.0));
        assert!(style.margin.left.is_some_and(f32::is_infinite));
        assert!(style.margin.right.is_some_and(f32::is_infinite));
        assert_eq!(style.bg, Some(Color32::from_rgba_premultiplied(0, 0, 0, 0)));
        assert_eq!(style.color.map(|color| color.a()), Some(128));
    }

    #[test]
    fn parses_alpha_hex_colors() {
        assert_eq!(
            parse_color("#11223344"),
            Some(Color32::from_rgba_premultiplied(17, 34, 51, 68))
        );
    }

    #[test]
    fn collects_subresource_manifest() {
        let src = "<html><head><link rel=\"stylesheet\" href=\"/a.css\"></head>\
                   <body><img src=\"/x.png\"><script src=\"/s.js\"></script></body></html>";
        let doc = HtmlDocument::parse(src);
        let manifest = doc.collect_subresources("https://example.com/base/index.html");
        assert_eq!(
            manifest.stylesheets,
            vec!["https://example.com/a.css".to_owned()]
        );
        assert_eq!(
            manifest.images,
            vec!["https://example.com/x.png".to_owned()]
        );
        assert_eq!(
            manifest.scripts,
            vec!["https://example.com/s.js".to_owned()]
        );
    }

    #[test]
    fn collects_legacy_image_alias_sources() {
        let src = "<html><body><image src=\"/legacy.png\"></body></html>";
        let doc = HtmlDocument::parse(src);
        let manifest = doc.collect_subresources("https://example.com/base/index.html");
        assert_eq!(
            manifest.images,
            vec!["https://example.com/legacy.png".to_owned()]
        );
    }

    #[test]
    fn collects_image_sources_from_srcset_and_source_tags() {
        let src = "<html><body>\
                   <img srcset=\"/logo-1x.png 1x, /logo-2x.png 2x\">\
                   <picture><source srcset=\"/hero.webp 1x, /hero@2x.webp 2x\"></picture>\
                   </body></html>";
        let doc = HtmlDocument::parse(src);
        let manifest = doc.collect_subresources("https://example.com/base/index.html");
        assert_eq!(
            manifest.images,
            vec![
                "https://example.com/hero.webp".to_owned(),
                "https://example.com/logo-1x.png".to_owned(),
            ]
        );
    }

    #[test]
    fn collects_only_executable_inline_scripts() {
        let src = "<html><body>\
                   <script>window.a=1;</script>\
                   <script type=\"application/ld+json\">{\"ok\":true}</script>\
                   <script type=\"module\">import x from '/x.js';</script>\
                   <script language=\"javascript\">window.b=2;</script>\
                   </body></html>";
        let doc = HtmlDocument::parse(src);
        let scripts = doc.collect_script_descriptors("https://example.com/");
        assert_eq!(scripts.len(), 2);
        match &scripts[0] {
            ScriptDescriptor::Inline { source } => assert!(source.contains("window.a=1")),
            _ => panic!("expected first script to be inline"),
        }
        match &scripts[1] {
            ScriptDescriptor::Inline { source } => assert!(source.contains("window.b=2")),
            _ => panic!("expected second script to be inline"),
        }
    }

    #[test]
    fn collects_script_descriptors_in_dom_order() {
        let src = "<html><body>\
                   <script>window.a=1;</script>\
                   <div><script src=\"/one.js\"></script></div>\
                   <script>window.b=2;</script>\
                   </body></html>";
        let doc = HtmlDocument::parse(src);
        let scripts = doc.collect_script_descriptors("https://example.com/base/index.html");
        assert_eq!(scripts.len(), 3);
        match &scripts[0] {
            ScriptDescriptor::Inline { source } => assert!(source.contains("window.a=1")),
            _ => panic!("expected inline script first"),
        }
        match &scripts[1] {
            ScriptDescriptor::External { url } => {
                assert_eq!(url, "https://example.com/one.js");
            }
            _ => panic!("expected external script second"),
        }
        match &scripts[2] {
            ScriptDescriptor::Inline { source } => assert!(source.contains("window.b=2")),
            _ => panic!("expected inline script third"),
        }
    }

    #[test]
    fn collects_id_elements_with_text_and_attributes() {
        let src = "<html><body>\
                   <div id=\"hero\" class=\"banner\">Hello <span>World</span></div>\
                   <input id=\"q\" name=\"q\" value=\"test\">\
                   </body></html>";
        let doc = HtmlDocument::parse(src);
        let nodes = doc.collect_id_elements(8);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, "hero");
        assert_eq!(nodes[0].tag_name, "DIV");
        assert!(nodes[0].text_content.contains("Hello World"));
        assert!(
            nodes[0]
                .attributes
                .iter()
                .any(|(name, value)| name == "class" && value == "banner")
        );
        assert_eq!(nodes[1].id, "q");
        assert_eq!(nodes[1].tag_name, "INPUT");
    }

    #[test]
    fn counts_inline_style_tags_and_rules() {
        let src = "<html><head><style>p{color:red}a{color:blue}</style></head><body></body></html>";
        let doc = HtmlDocument::parse(src);
        assert_eq!(doc.inline_style_tag_count(), 1);
        assert_eq!(doc.css_rule_count(), 2);
    }

    #[test]
    fn cascade_prefers_higher_specificity() {
        let sheet = StyleSheet {
            rules: parse_css_rules(
                "div { color: #101010; } .card { color: #202020; } #hero { color: #303030; }",
            ),
        };

        let el = HtmlElement {
            tag: "div".to_owned(),
            attrs: vec![
                ("id".to_owned(), "hero".to_owned()),
                ("class".to_owned(), "card".to_owned()),
            ],
            children: Vec::new(),
        };

        let style = style_for(&el, &sheet, &StyleProps::default(), &[]);
        assert_eq!(style.color, Some(Color32::from_rgb(48, 48, 48)));
    }

    #[test]
    fn cascade_prefers_important_over_inline() {
        let sheet = StyleSheet {
            rules: parse_css_rules("#hero { color: #ff0000 !important; }"),
        };

        let el = HtmlElement {
            tag: "div".to_owned(),
            attrs: vec![
                ("id".to_owned(), "hero".to_owned()),
                ("style".to_owned(), "color: #0000ff;".to_owned()),
            ],
            children: Vec::new(),
        };

        let style = style_for(&el, &sheet, &StyleProps::default(), &[]);
        assert_eq!(style.color, Some(Color32::from_rgb(255, 0, 0)));
    }

    #[test]
    fn inherit_keyword_resets_to_parent_value() {
        let sheet = StyleSheet {
            rules: parse_css_rules(".muted { color: #999999; } .reset { color: inherit; }"),
        };

        let el = HtmlElement {
            tag: "span".to_owned(),
            attrs: vec![("class".to_owned(), "muted reset".to_owned())],
            children: Vec::new(),
        };

        let inherited = StyleProps {
            color: Some(Color32::from_rgb(4, 120, 78)),
            ..StyleProps::default()
        };

        let style = style_for(&el, &sheet, &inherited, &[]);
        assert_eq!(style.color, Some(Color32::from_rgb(4, 120, 78)));
    }

    #[test]
    fn visibility_inherits_and_opacity_composes() {
        let sheet = StyleSheet::default();
        let inherited = StyleProps {
            visibility_hidden: Some(true),
            opacity: Some(0.5),
            ..StyleProps::default()
        };

        let plain = HtmlElement {
            tag: "div".to_owned(),
            attrs: Vec::new(),
            children: Vec::new(),
        };
        let plain_style = style_for(&plain, &sheet, &inherited, &[]);
        assert_eq!(plain_style.visibility_hidden, Some(true));
        assert_eq!(plain_style.opacity, Some(0.5));

        let overridden = HtmlElement {
            tag: "div".to_owned(),
            attrs: vec![(
                "style".to_owned(),
                "visibility:visible;opacity:0.2;".to_owned(),
            )],
            children: Vec::new(),
        };
        let overridden_style = style_for(&overridden, &sheet, &inherited, &[]);
        assert_eq!(overridden_style.visibility_hidden, Some(false));
        assert!(
            overridden_style
                .opacity
                .is_some_and(|value| (value - 0.1).abs() < 0.0001)
        );
    }

    #[test]
    fn raw_css_property_cascade_prefers_important() {
        let sheet = StyleSheet {
            rules: parse_css_rules(
                ".pane { scrollbar-gutter: auto; } #hero { scrollbar-gutter: stable !important; }",
            ),
        };

        let el = HtmlElement {
            tag: "div".to_owned(),
            attrs: vec![
                ("id".to_owned(), "hero".to_owned()),
                ("class".to_owned(), "pane".to_owned()),
                ("style".to_owned(), "scrollbar-gutter:always".to_owned()),
            ],
            children: Vec::new(),
        };

        let style = style_for(&el, &sheet, &StyleProps::default(), &[]);
        assert_eq!(
            style.raw_css.get("scrollbar-gutter").map(String::as_str),
            Some("stable")
        );
    }

    #[test]
    fn script_raw_text_is_not_tokenized_as_dom_nodes() {
        let src = "<body><script>var t = '<div>bad</div>';</script><p>ok</p></body>";
        let doc = HtmlDocument::parse(src);
        let visible = collect_visible_text(&doc.root.children);
        assert!(visible.contains("ok"));
        assert!(!visible.contains("bad"));
        assert!(!visible.contains("var t"));
    }

    #[test]
    fn decodes_numeric_entities() {
        assert_eq!(decode_entities("&#1589;&#1601;"), "صف");
        assert_eq!(decode_entities("&#x41;&#X42;"), "AB");
    }

    #[test]
    fn finds_body_element() {
        let src = "<html><head><title>X</title></head><body><p>ok</p></body></html>";
        let doc = HtmlDocument::parse(src);
        let body = find_first_element(&doc.root.children, "body");
        assert!(body.is_some());
    }

    #[test]
    fn dir_rtl_sets_default_alignment() {
        let sheet = StyleSheet::default();
        let el = HtmlElement {
            tag: "div".to_owned(),
            attrs: vec![("dir".to_owned(), "rtl".to_owned())],
            children: Vec::new(),
        };
        let style = style_for(&el, &sheet, &StyleProps::default(), &[]);
        assert_eq!(style.text_align, Some(TextAlign::Right));
    }

    #[test]
    fn html_presentational_attributes_map_into_style() {
        let sheet = StyleSheet::default();
        let el = HtmlElement {
            tag: "font".to_owned(),
            attrs: vec![
                ("align".to_owned(), "center".to_owned()),
                ("bgcolor".to_owned(), "#101112".to_owned()),
                ("color".to_owned(), "#f0f0f0".to_owned()),
                ("width".to_owned(), "50%".to_owned()),
                ("height".to_owned(), "120".to_owned()),
                ("border".to_owned(), "2".to_owned()),
                ("face".to_owned(), "Courier New".to_owned()),
                ("size".to_owned(), "5".to_owned()),
            ],
            children: Vec::new(),
        };
        let style = style_for(&el, &sheet, &StyleProps::default(), &[]);
        assert_eq!(style.text_align, Some(TextAlign::Center));
        assert_eq!(style.bg, Some(Color32::from_rgb(16, 17, 18)));
        assert_eq!(style.color, Some(Color32::from_rgb(240, 240, 240)));
        assert_eq!(style.width_percent, Some(50.0));
        assert_eq!(style.height, Some(120.0));
        assert_eq!(style.border_width.top, Some(2.0));
        assert_eq!(style.font_family, Some(FontFamilyChoice::Monospace));
        assert_eq!(style.font_size, Some(24.0));
    }

    #[test]
    fn parses_legacy_font_size_values() {
        assert_eq!(parse_legacy_font_size("1"), Some(10.0));
        assert_eq!(parse_legacy_font_size("3"), Some(16.0));
        assert_eq!(parse_legacy_font_size("7"), Some(48.0));
        assert_eq!(parse_legacy_font_size("+2"), Some(24.0));
        assert_eq!(parse_legacy_font_size("-1"), Some(13.0));
    }

    #[test]
    fn formats_list_markers_from_type_attributes() {
        assert_eq!(ordered_list_marker(3, Some("1")), "3.");
        assert_eq!(ordered_list_marker(3, Some("A")), "C.");
        assert_eq!(ordered_list_marker(3, Some("a")), "c.");
        assert_eq!(ordered_list_marker(4, Some("I")), "IV.");
        assert_eq!(ordered_list_marker(4, Some("i")), "iv.");
        assert_eq!(unordered_list_marker(Some("disc")), "\u{2022}");
        assert_eq!(unordered_list_marker(Some("circle")), "\u{25E6}");
        assert_eq!(unordered_list_marker(Some("square")), "\u{25AA}");
    }

    #[test]
    fn collects_background_and_video_poster_resources() {
        let src = "<html><body background=\"/bg.png\"><video poster=\"/poster.jpg\"></video></body></html>";
        let doc = HtmlDocument::parse(src);
        let manifest = doc.collect_subresources("https://example.com/base/index.html");
        assert_eq!(
            manifest.images,
            vec![
                "https://example.com/bg.png".to_owned(),
                "https://example.com/poster.jpg".to_owned(),
            ]
        );
    }

    #[test]
    fn complex_selector_requires_matching_ancestor_context() {
        let sheet = StyleSheet {
            rules: parse_css_rules(".scope #hero .item { color: #123456; }"),
        };

        let el = HtmlElement {
            tag: "span".to_owned(),
            attrs: vec![("class".to_owned(), "item".to_owned())],
            children: Vec::new(),
        };

        let style = style_for(&el, &sheet, &StyleProps::default(), &[]);
        assert_eq!(style.color, None);
    }

    #[test]
    fn complex_selector_matches_descendant_chain() {
        let sheet = StyleSheet {
            rules: parse_css_rules(".scope #hero .item { color: #123456; }"),
        };

        let scope = HtmlElement {
            tag: "div".to_owned(),
            attrs: vec![("class".to_owned(), "scope".to_owned())],
            children: Vec::new(),
        };
        let hero = HtmlElement {
            tag: "section".to_owned(),
            attrs: vec![("id".to_owned(), "hero".to_owned())],
            children: Vec::new(),
        };
        let el = HtmlElement {
            tag: "span".to_owned(),
            attrs: vec![("class".to_owned(), "item".to_owned())],
            children: Vec::new(),
        };

        let ancestors = vec![selector_subject(&scope), selector_subject(&hero)];
        let style = style_for(&el, &sheet, &StyleProps::default(), &ancestors);
        assert_eq!(style.color, Some(Color32::from_rgb(18, 52, 86)));
    }

    #[test]
    fn child_combinator_matches_direct_parent_only() {
        let sheet = StyleSheet {
            rules: parse_css_rules("div > .item { color: #010203; }"),
        };

        let parent = HtmlElement {
            tag: "div".to_owned(),
            attrs: Vec::new(),
            children: Vec::new(),
        };
        let el = HtmlElement {
            tag: "span".to_owned(),
            attrs: vec![("class".to_owned(), "item".to_owned())],
            children: Vec::new(),
        };

        let direct_ancestors = vec![selector_subject(&parent)];
        let style = style_for(&el, &sheet, &StyleProps::default(), &direct_ancestors);
        assert_eq!(style.color, Some(Color32::from_rgb(1, 2, 3)));

        let non_matching = HtmlElement {
            tag: "section".to_owned(),
            attrs: Vec::new(),
            children: Vec::new(),
        };
        let wrong_ancestors = vec![selector_subject(&parent), selector_subject(&non_matching)];
        let wrong_style = style_for(&el, &sheet, &StyleProps::default(), &wrong_ancestors);
        assert_eq!(wrong_style.color, None);
    }

    #[test]
    fn unsupported_pseudo_selectors_do_not_overmatch() {
        let sheet = StyleSheet {
            rules: parse_css_rules(".card:not(.active) { display:none; }"),
        };

        let el = HtmlElement {
            tag: "div".to_owned(),
            attrs: vec![("class".to_owned(), "card active".to_owned())],
            children: Vec::new(),
        };

        let style = style_for(&el, &sheet, &StyleProps::default(), &[]);
        assert_eq!(style.display, None);
    }

    #[test]
    fn unsupported_attribute_selectors_do_not_overmatch() {
        let sheet = StyleSheet {
            rules: parse_css_rules(".card[data-state='hidden'] { display:none; }"),
        };

        let el = HtmlElement {
            tag: "div".to_owned(),
            attrs: vec![("class".to_owned(), "card".to_owned())],
            children: Vec::new(),
        };

        let style = style_for(&el, &sheet, &StyleProps::default(), &[]);
        assert_eq!(style.display, None);
    }

    fn collect_visible_text(nodes: &[HtmlNode]) -> String {
        let mut out = String::new();
        for node in nodes {
            match node {
                HtmlNode::Text(text) => out.push_str(text),
                HtmlNode::Element(el) => {
                    if matches!(el.tag.as_str(), "script" | "style" | "noscript") {
                        continue;
                    }
                    out.push_str(&collect_visible_text(&el.children));
                }
            }
        }
        out
    }
}
