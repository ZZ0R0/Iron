//! [20200] Inférence Option/Result depuis le flux (I11–I13).
//!
//! Branché à l'incrément 3. Détecte les chemins « sans valeur » → `Option`, et l'usage de
//! `?` / opérations faillibles → `Result<_, IronError>`. Pour l'instant : no-op (les fonctions
//! portent un type de retour annoté).

use super::Inf;
use crate::ir::*;

pub(crate) fn annotate_returns(_cx: &mut Inf, _body: &[Stmt]) {
    // TODO incrément 3 : déduire Option/Result du flux et raffiner `cx.ret`.
}
