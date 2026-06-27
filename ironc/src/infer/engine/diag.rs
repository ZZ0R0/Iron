//! [22200] Diagnostics de type Iron (V10) : traduit un `TypeError` du solveur en `CompileError` porté par
//! le **span Iron** (I8/D8). Iron diagnostique enfin ses types lui-même, avant `rustc` (qui reste le filet,
//! D7). Le message suit le style de la remontée existante (`fichier:ligne:col` ajouté par le driver).

#![allow(dead_code)]

use super::solve::TypeError;
use crate::ir::CompileError;

pub fn to_compile_error(e: TypeError) -> CompileError {
    CompileError { span: Some(e.span), msg: format!("type : {}", e.msg) }
}
