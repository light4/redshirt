// Copyright(c) 2019 Pierre Krieger

//! Parsing of the XML definitions file.

use std::{collections::HashMap, io::Read};
use xml::{EventReader, attribute::OwnedAttribute, name::OwnedName, reader::Events, reader::XmlEvent};

/// Successfully-parsed Vulkan registry definitions.
///
/// > **Note**: This only contains the information we need. No need to completely parse
/// >           everything.
#[derive(Debug, Clone)]
pub struct VkRegistry {
    /// List of all the Vulkan commands.
    pub commands: Vec<VkCommand>,
    /// Type definitions.
    pub type_defs: HashMap<String, VkTypeDef>,
}

/// A type definition of the Vulkan API.
#[derive(Debug, Clone)]
pub enum VkTypeDef {
    Enum,
    Bitmask,
    Handle,
    Struct {
        fields: Vec<(VkType, String)>,
    },
}

/// Successfully-parsed Vulkan command definition.
#[derive(Debug, Clone)]
pub struct VkCommand {
    /// Name of the Vulkan function, with the `vk` prefix.
    pub name: String,
    /// Return type of the function.
    pub ret_ty: VkType,
    /// List of parameters of the function, with their type and name.
    pub params: Vec<(VkType, String)>
}

/// Successfully-parsed Vulkan type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VkType {
    Ident(String),
    /// Pointer to some memory location containing a certain number of elements of the given type.
    MutPointer(Box<VkType>, VkTypePtrLen),
    /// Pointer to some memory location containing a certain number of elements of the given type.
    ConstPointer(Box<VkType>, VkTypePtrLen),
    Array(Box<VkType>, String),
}

/// Number of elements in a memory location indicated with a pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VkTypePtrLen {
    One,
    NullTerminated,
    OtherField(String),
}

/// Parses the file `vk.xml` from the given source. Assumes that everything is well-formed and
/// that no error happens.
pub fn parse(source: impl Read) -> VkRegistry {
    let mut events_source = EventReader::new(source).into_iter();

    match events_source.next() {
        Some(Ok(XmlEvent::StartDocument { .. })) => {},
        ev => panic!("Unexpected: {:?}", ev)
    }

    let registry = match events_source.next() {
        Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "registry") =>
            parse_registry(&mut events_source),
        ev => panic!("Unexpected: {:?}", ev)
    };

    loop {
        match events_source.next() {
            Some(Ok(XmlEvent::EndDocument { .. })) => break,
            Some(Ok(XmlEvent::Whitespace(..))) => {},
            ev => panic!("Unexpected: {:?}", ev)
        }
    }

    match events_source.next() {
        None => return registry,
        ev => panic!("Unexpected: {:?}", ev)
    }
}

// # About parsing
//
// The XML library we're using proposes a streaming compilation API. What this means it that it
// parses the XML code and feeds us with parsing events such as `StartElement`, `EndElement`
// or `Characters`.
//
// The content of this module accomodates this. The various functions below expect as input
// a `&mut Events` (where `Events` is an iterator) and advance the iterator until they leave
// the current element. If anything unexpected is encountered on the way, everything stops and a
// panic immediately happens.
//

fn parse_registry(events_source: &mut Events<impl Read>) -> VkRegistry {
    let mut out = VkRegistry {
        commands: Vec::new(),
        type_defs: HashMap::new(),
    };

    loop {
        match events_source.next() {
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "types") => {
                let type_defs = parse_types(events_source);
                assert!(out.type_defs.is_empty());
                out.type_defs = type_defs;
            },
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "commands") => {
                let commands = parse_commands(events_source);
                assert!(out.commands.is_empty());
                out.commands = commands;
            },

            // We actually don't care what enum values are.
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "enums") =>
                advance_until_elem_end(events_source, &name),

            // Other things we don't care about.
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "comment") =>
                advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "platforms") =>
                advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "tags") =>
                advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "feature") =>
                advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "extensions") =>
                advance_until_elem_end(events_source, &name),

            Some(Ok(XmlEvent::EndElement { .. })) => {
                assert!(!out.commands.is_empty());
                assert!(!out.type_defs.is_empty());
                return out;
            },
            Some(Ok(XmlEvent::CData(..))) |
            Some(Ok(XmlEvent::Comment(..))) |
            Some(Ok(XmlEvent::Characters(..))) |
            Some(Ok(XmlEvent::Whitespace(..))) => {},
            ev => panic!("Unexpected; probably because unimplemented: {:?}", ev),      // TODO: turn into "Unexpected" once everything is implemented
        }
    }
}

/// Call this function right after finding a `StartElement` with the name `types`. This function
/// parses the content of the element.
fn parse_types(events_source: &mut Events<impl Read>) -> HashMap<String, VkTypeDef> {
    let mut out = HashMap::new();

    loop {
        match events_source.next() {
            Some(Ok(XmlEvent::StartElement { name, attributes, .. })) if name_equals(&name, "type") => {
                if let Some((name, ty)) = parse_type(events_source, attributes) {
                    if !name.is_empty() {        // TODO: shouldn't be there; find the bug
                        let _prev_val = out.insert(name.clone(), ty);
                        assert!(_prev_val.is_none(), "Duplicate value for {:?}", name);
                    }
                }
            },
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "comment") =>
                advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::EndElement { name, .. })) => {
                assert!(name_equals(&name, "types"));
                return out
            },
            Some(Ok(XmlEvent::CData(..))) |
            Some(Ok(XmlEvent::Comment(..))) |
            Some(Ok(XmlEvent::Characters(..))) |
            Some(Ok(XmlEvent::Whitespace(..))) => {},
            ev => panic!("Unexpected: {:?}", ev),
        }
    }
}

/// Call this function right after finding a `StartElement` with the name `type`. This
/// function parses the content of the element.
fn parse_type(events_source: &mut Events<impl Read>, attributes: Vec<OwnedAttribute>) -> Option<(String, VkTypeDef)> {
    match find_attr(&attributes, "category") {
        Some("enum") => {
            let name = find_attr(&attributes, "name").unwrap().to_owned();
            advance_until_elem_end(events_source, &"type".parse().unwrap());
            Some((name, VkTypeDef::Enum))
        },
        Some("bitmask") => {
            let (_, name) = parse_ty_name(events_source, attributes);
            Some((name, VkTypeDef::Bitmask))
        },
        Some("include") | Some("define") | Some("basetype") => {
            advance_until_elem_end(events_source, &"type".parse().unwrap());
            None
        },
        Some("handle") => {
            let (_, name) = parse_ty_name(events_source, attributes);
            Some((name, VkTypeDef::Handle))
        },
        Some("funcpointer") => {
            // We deliberately ignore function pointers, and manually generate their definitions.
            advance_until_elem_end(events_source, &"type".parse().unwrap());
            None
        },
        Some("union") => {
            advance_until_elem_end(events_source, &"type".parse().unwrap());
            None      // TODO: wrong
        },
        Some("struct") => {
            let name = find_attr(&attributes, "name").unwrap().to_owned();
            let mut fields = Vec::new();

            loop {
                match events_source.next() {
                    Some(Ok(XmlEvent::StartElement { name, attributes, .. })) if name_equals(&name, "member") =>{
                        fields.push(parse_ty_name(events_source, attributes));
                    },
                    Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "comment") =>
                        advance_until_elem_end(events_source, &name),
                    Some(Ok(XmlEvent::EndElement { .. })) => break,
                    Some(Ok(XmlEvent::CData(..))) |
                    Some(Ok(XmlEvent::Comment(..))) |
                    Some(Ok(XmlEvent::Characters(..))) |
                    Some(Ok(XmlEvent::Whitespace(..))) => {},
                    ev => panic!("Unexpected: {:?}", ev),
                }
            }

            Some((name, VkTypeDef::Struct { fields }))
        },
        None if find_attr(&attributes, "requires").is_some() => {
            advance_until_elem_end(events_source, &"type".parse().unwrap());
            None
        },
        None if find_attr(&attributes, "name") == Some("int") => {
            advance_until_elem_end(events_source, &"type".parse().unwrap());
            None
        },
        cat => panic!("Unexpected type category: {:?} with attrs {:?}", cat, attributes),
    }
}

/// Call this function right after finding a `StartElement` with the name `commands`. This
/// function parses the content of the element.
fn parse_commands(events_source: &mut Events<impl Read>) -> Vec<VkCommand> {
    let mut out = Vec::new();

    loop {
        match events_source.next() {
            Some(Ok(XmlEvent::StartElement { name, attributes, .. })) if name_equals(&name, "command") => {
                if let Some(cmd) = parse_command(events_source, attributes) {
                    out.push(cmd);
                }
            },
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "comment") =>
                advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::EndElement { .. })) => return out,
            Some(Ok(XmlEvent::CData(..))) |
            Some(Ok(XmlEvent::Comment(..))) |
            Some(Ok(XmlEvent::Characters(..))) |
            Some(Ok(XmlEvent::Whitespace(..))) => {},
            ev => panic!("Unexpected: {:?}", ev),
        }
    }
}

/// Call this function right after finding a `StartElement` with the name `command`. This
/// function parses the content of the element.
fn parse_command(events_source: &mut Events<impl Read>, attributes: Vec<OwnedAttribute>) -> Option<VkCommand> {
    let mut out = VkCommand {
        name: String::new(),
        ret_ty: VkType::Ident(String::new()),
        params: Vec::new(),
    };

    loop {
        match events_source.next() {
            Some(Ok(XmlEvent::StartElement { name, attributes, .. })) if name_equals(&name, "proto") => {
                let (ret_ty, f_name) = parse_ty_name(events_source, attributes);
                out.name = f_name;
                out.ret_ty = ret_ty;
            },

            Some(Ok(XmlEvent::StartElement { name, attributes, .. })) if name_equals(&name, "param") =>{
                out.params.push(parse_ty_name(events_source, attributes));
            },

            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "implicitexternsyncparams") =>
                advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::EndElement { .. })) => break,
            Some(Ok(XmlEvent::CData(..))) |
            Some(Ok(XmlEvent::Comment(..))) |
            Some(Ok(XmlEvent::Characters(..))) |
            Some(Ok(XmlEvent::Whitespace(..))) => {},
            ev => panic!("Unexpected: {:?}", ev),
        }
    }

    if out.name.is_empty() || out.ret_ty == VkType::Ident(String::new()) {
        // TODO: aliases must also be returned somehow
        assert!(find_attr(&attributes, "alias").is_some());
        return None;
    }

    Some(out)
}

/// Call this function right after finding a `StartElement`. This function parses the content of
/// the element and expects a single `<type>` tag and a single `<name>` tag. It returns the type
/// and the name.
fn parse_ty_name(events_source: &mut Events<impl Read>, attributes: Vec<OwnedAttribute>) -> (VkType, String) {
    let mut ret_ty_out = String::new();
    let mut name_out = String::new();
    let mut enum_content = String::new();
    let len_attr = find_attr(&attributes, "len");

    let mut white_spaces = String::new();

    loop {
        match events_source.next() {
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "name") =>
                name_out = expect_characters_elem(events_source),
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "type") =>
                ret_ty_out = expect_characters_elem(events_source),
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "enum") =>
                enum_content = expect_characters_elem(events_source),
            Some(Ok(XmlEvent::StartElement { name, .. })) if name_equals(&name, "comment") =>
                advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::EndElement { .. })) => break,
            Some(Ok(XmlEvent::CData(s))) => white_spaces.push_str(&s),
            Some(Ok(XmlEvent::Comment(s))) => white_spaces.push_str(&s),
            Some(Ok(XmlEvent::Characters(s))) => white_spaces.push_str(&s),
            Some(Ok(XmlEvent::Whitespace(s))) => white_spaces.push_str(&s),
            ev => panic!("Unexpected: {:?}", ev),
        }
    }

    // TODO: there's also "**"
    let ret_ty = if white_spaces.contains("*") {
        let len = if let Some(len) = len_attr {
            if len == "null-terminated" {
                VkTypePtrLen::NullTerminated
            } else {
                VkTypePtrLen::OtherField(len.to_owned())
            }
            // TODO: len.split();   number of elements gives nesting level
        } else {
            VkTypePtrLen::One
        };

        if white_spaces.contains("const") {
            VkType::ConstPointer(Box::new(VkType::Ident(ret_ty_out)), len)
        } else {
            VkType::MutPointer(Box::new(VkType::Ident(ret_ty_out)), len)
        }

    } else {
        assert!(len_attr.is_none());

        if white_spaces.contains("[") {
            if enum_content.is_empty() {
                // TODO: hard-coded :-/
                if white_spaces.contains("[2]") {
                    VkType::Array(Box::new(VkType::Ident(ret_ty_out)), "2".into())
                } else if white_spaces.contains("[3]") {
                    VkType::Array(Box::new(VkType::Ident(ret_ty_out)), "3".into())
                } else if white_spaces.contains("[4]") {
                    VkType::Array(Box::new(VkType::Ident(ret_ty_out)), "4".into())
                } else {
                    panic!()
                }
            } else {
                VkType::Array(Box::new(VkType::Ident(ret_ty_out)), enum_content)
            }
        } else {
            VkType::Ident(ret_ty_out)
        }
    };

    (ret_ty, name_out)
}

/// Advances the `events_source` until a corresponding `EndElement` with the given `elem` is found.
///
/// Call this function if you find a `StartElement` whose content you don't care about.
fn advance_until_elem_end(events_source: &mut Events<impl Read>, elem: &OwnedName) {
    loop {
        match events_source.next() {
            Some(Ok(XmlEvent::StartElement { name, .. })) => advance_until_elem_end(events_source, &name),
            Some(Ok(XmlEvent::EndElement { name })) if &name == elem => return,
            Some(Ok(XmlEvent::CData(..))) |
            Some(Ok(XmlEvent::Comment(..))) |
            Some(Ok(XmlEvent::Characters(..))) |
            Some(Ok(XmlEvent::Whitespace(..))) => {},
            ev => panic!("Unexpected: {:?}", ev),
        }
    }
}

/// Call this function if you find a `StartElement`. This function will grab any character within
/// the element and will return when it encounters the corresponding `EndElement`. Any other
/// `StartElement` within will trigger a panic.
fn expect_characters_elem(events_source: &mut Events<impl Read>) -> String {
    let mut out = String::new();

    loop {
        match events_source.next() {
            Some(Ok(XmlEvent::EndElement { .. })) => return out,
            Some(Ok(XmlEvent::CData(s))) => out.push_str(&s),
            Some(Ok(XmlEvent::Comment(s))) => out.push_str(&s),
            Some(Ok(XmlEvent::Characters(s))) => out.push_str(&s),
            Some(Ok(XmlEvent::Whitespace(s))) => out.push_str(&s),
            ev => panic!("Unexpected: {:?}", ev),
        }
    }
}

/// Checks whether an `OwnedName` matches the expected name.
fn name_equals(name: &OwnedName, expected: &str) -> bool {
    name.namespace.is_none() && name.prefix.is_none() && name.local_name == expected
}

/// Find an attribute value in the list.
fn find_attr<'a>(list: &'a [OwnedAttribute], name: &str) -> Option<&'a str> {
    list.iter().find(|a| name_equals(&a.name, name)).map(|a| a.value.as_str())
}