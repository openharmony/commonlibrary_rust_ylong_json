// Copyright (c) 2023 Huawei Device Co., Ltd.
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

#[cfg(feature = "btree_object")]
mod btree;
#[cfg(feature = "btree_object")]
pub use btree::Object;

#[cfg(feature = "list_object")]
mod linked_list;
#[cfg(feature = "list_object")]
pub use linked_list::Object;

#[cfg(feature = "vec_object")]
mod vec;
#[cfg(feature = "vec_object")]
pub use vec::Object;
