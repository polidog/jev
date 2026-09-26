mod cloudflare;
mod typesafe;
mod vercel;

use crate::cli::ProviderKind;
use crate::model::{Request, Response};

/// A backend serving Jev. Providers translate to and from TypeSafe's wire types.
pub trait Provider {
    fn evaluate(&self, req: &Request) -> anyhow::Result<Response>;
}

pub fn of(kind: ProviderKind) -> &'static dyn Provider {
    match kind {
        ProviderKind::Typesafe => &typesafe::TypeSafe,
        ProviderKind::Cloudflare => &cloudflare::Cloudflare,
        ProviderKind::Vercel => &vercel::Vercel,
    }
}
