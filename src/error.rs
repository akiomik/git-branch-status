// Copyright 2021 Akiomi Kamakura
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::absolute_paths)]

/// The single domain error for the `gix` backend.
///
/// The many per-operation `gix` error types (some of which are large) are boxed
/// behind this one type, so the rest of the crate never names a `gix` type and
/// the `Result` stays small.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(Box<dyn std::error::Error + Send + Sync + 'static>);

macro_rules! impl_from_gix_error {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl From<$ty> for Error {
                fn from(err: $ty) -> Self {
                    Error(Box::new(err))
                }
            }
        )+
    };
}

impl_from_gix_error!(
    gix::discover::Error,
    gix::reference::find::existing::Error,
    gix::status::Error,
    gix::status::into_iter::Error,
    gix::status::iter::Error,
);
