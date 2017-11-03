//! Functions common for all target languages.

use std::path::PathBuf;
use std::collections::HashMap;
use syntax::ast;
use syntax::print::pprust;

use Error;
use Level;

/// Outputs several files as a result of an AST transformation.
pub type Outputs = HashMap<PathBuf, String>;

/// Target language support
pub trait Lang {
    /// Convert `pub type A = B;` into `typedef B A;`.
    fn parse_ty(_item: &ast::Item) -> Result<Option<Outputs>, Error> {
        Ok(None)
    }

    /// Convert a Rust enum into a target language enum.
    fn parse_enum(_item: &ast::Item) -> Result<Option<Outputs>, Error> {
        Ok(None)
    }

    /// Convert a Rust struct into a target language struct.
    fn parse_struct(_item: &ast::Item) -> Result<Option<Outputs>, Error> {
        Ok(None)
    }

    /// Convert a Rust function declaration into a target language function declaration.
    fn parse_fn(_item: &ast::Item) -> Result<Option<Outputs>, Error> {
        Ok(None)
    }
}

/// Check the attribute is #[no_mangle].
pub fn check_no_mangle(attr: &ast::Attribute) -> bool {
    match attr.value.node {
        ast::MetaItemKind::Word if attr.name() == "no_mangle" => true,
        _ => false,
    }
}

/// Check the function argument is `user_data: *mut c_void`
pub fn is_user_data_arg(arg: &ast::Arg) -> bool {
    pprust::pat_to_string(&*arg.pat) == "user_data" &&
        pprust::ty_to_string(&*arg.ty) == "*mut c_void"
}

/// Check the function argument is `result: *const FfiResult`
pub fn is_result_arg(arg: &ast::Arg) -> bool {
    pprust::pat_to_string(&*arg.pat) == "result" &&
        pprust::ty_to_string(&*arg.ty) == "*const FfiResult"
}

/// Transform function arguments into a (name, type) pair
pub fn fn_args(inputs: &Vec<ast::Arg>, name: &str) -> Result<Vec<(String, ast::Ty)>, Error> {
    inputs
        .iter()
        .map(|ref arg| {
            use syntax::ast::{PatKind, BindingMode};
            let arg_name = match arg.pat.node {
                PatKind::Ident(BindingMode::ByValue(_), ref ident, None) => {
                    ident.node.name.to_string()
                }
                _ => {
                    return Err(Error {
                        level: Level::Error,
                        span: None,
                        message: format!(
                            "cheddar only supports by-value arguments:
    incorrect argument `{}` in function definition `{}`",
                            pprust::pat_to_string(&*arg.pat),
                            name
                        ),
                    })
                }
            };
            let arg_ty: &ast::Ty = &*arg.ty.clone();
            Ok((arg_name, arg_ty.clone()))
        })
        .collect()
}

// TODO: Maybe it would be wise to use syntax::attr here.
/// Loop through a list of attributes.
///
/// Check that at least one attribute matches some criteria (usually #[repr(C)] or #[no_mangle])
/// and optionally retrieve a String from it (usually a docstring).
pub fn parse_attr<C, R>(attrs: &[ast::Attribute], check: C, retrieve: R) -> (bool, String)
where
    C: Fn(&ast::Attribute) -> bool,
    R: Fn(&ast::Attribute) -> Option<String>,
{
    let mut check_passed = false;
    let mut retrieved_str = String::new();
    for attr in attrs {
        // Don't want to accidently set it to false after it's been set to true.
        if !check_passed {
            check_passed = check(attr);
        }
        // If this attribute has any strings to retrieve, retrieve them.
        if let Some(string) = retrieve(attr) {
            retrieved_str.push_str(&string);
        }
    }

    (check_passed, retrieved_str)
}

/// Check the attribute is #[repr(C)].
pub fn check_repr_c(attr: &ast::Attribute) -> bool {
    match attr.value.node {
        ast::MetaItemKind::List(ref word) if attr.name() == "repr" => {
            match word.first() {
                Some(word) => {
                    match word.node {
                        // Return true only if attribute is #[repr(C)].
                        ast::NestedMetaItemKind::MetaItem(ref item) if item.name == "C" => true,
                        _ => false,
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// If the attribute is  a docstring, indent it the required amount and return it.
pub fn retrieve_docstring(attr: &ast::Attribute, prepend: &str) -> Option<String> {
    match attr.value.node {
        ast::MetaItemKind::NameValue(ref val) if attr.name() == "doc" => {
            match val.node {
                // Docstring attributes omit the trailing newline.
                ast::LitKind::Str(ref docs, _) => Some(format!("{}{}\n", prepend, docs)),
                _ => unreachable!("docs must be literal strings"),
            }
        }
        _ => None,
    }
}
