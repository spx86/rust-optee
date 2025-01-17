#!/bin/bash

authors="spengxu"

# 检查是否提供了项目名称
if [ -z "$1" ]; then
  echo "Usage: $0 <project_name>"
  exit 1
fi

PROJECT_NAME=$1
BASE_DIR="$(pwd)/$PROJECT_NAME"

# 创建项目根目录
mkdir -p "$BASE_DIR"
cd "$BASE_DIR" || exit 1
cat <<EOF > Cargo.toml
[workspace]

resolver = "2"

[profile.release]
panic = "abort"
EOF

# 创建顶层 Makefile
cat <<EOF > Makefile
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

all:
	make -C host 
	make -C ta 

install: ta
	make -C ta install

run-only:
	make -C host run

clean:
	make -C host clean
	make -C ta clean
EOF

# 创建 host 项目
mkdir -p host/src
cd host || exit 1
cargo init --bin
cat <<EOF > Makefile
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

NAME := $PROJECT_NAME
ARCH ?= aarch64

TARGET ?= aarch64-unknown-linux-gnu
OBJCOPY := objcopy

OUT_DIR := \$(CURDIR)/../target/\$(TARGET)/release


all: host strip

host:
	@cargo build --target \$(TARGET) --release

fmt:
	@cargo fmt

strip: host
	@\$(OBJCOPY) --strip-unneeded \$(OUT_DIR)/\$(NAME) \$(OUT_DIR)/\$(NAME)

clean:
	@cargo clean

run:
	\$(OUT_DIR)/\$(NAME)

EOF

cat <<EOF > ./src/main.rs
// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use optee_teec::{Context, Operation, ParamType, Session, Uuid};
use optee_teec::{ParamNone, ParamValue};
use proto::{UUID, Command};

fn hello_world(session: &mut Session) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(29, 0, ParamType::ValueInout);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    println!("original value is {:?}", operation.parameters().0.a());

    session.invoke_command(Command::IncValue as u32, &mut operation)?;
    println!("inc value is {:?}", operation.parameters().0.a());

    session.invoke_command(Command::DecValue as u32, &mut operation)?;
    println!("dec value is {:?}", operation.parameters().0.a());
    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID).unwrap();
    let mut session = ctx.open_session(uuid)?;

    hello_world(&mut session)?;

    println!("Success");
    Ok(())
}
EOF

# 在 host/Cargo.toml 中添加依赖项

sed -i '/\[dependencies\]/a proto = { path = "../proto" } \noptee-teec = { git = "https://github.com/apache/incubator-teaclave-trustzone-sdk.git", branch = "main", default-features = false }\n' ./Cargo.toml

sed -i '$a [profile.release]\nlto = true\n' ./Cargo.toml

# 修改 ta/Cargo.toml 中的版本和作者
sed -i "s/^authors = .*/authors = [\"$authors\"]/" ./Cargo.toml
sed -i "s/^name = .*/name = \"$PROJECT_NAME\" /" ./Cargo.toml

cd ..

# 创建 proto 项目
mkdir -p proto/src
cd proto || exit 1
cargo init --lib
cat <<EOF > build.rs
use std::fs;
use std::path::PathBuf;
use std::fs::File;
use std::env;
use std::io::Write;

fn main() {
    let uuid = match fs::read_to_string("../uuid.txt") {
        Ok(u) => {
            u.trim().to_string()
        },
        Err(_) => {
            panic!("Cannot find uuid.txt");
        }
    };
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut buffer = File::create(out.join("uuid.txt")).unwrap();
    write!(buffer, "{}", uuid).unwrap();
}
EOF

cat <<EOF > ./src/lib.rs
// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![no_std]
use num_enum::{TryFromPrimitive, IntoPrimitive};

#[derive(TryFromPrimitive, IntoPrimitive, Debug, Copy, Clone)]
#[repr(u32)]
pub enum Command {
    IncValue,
    DecValue,
    Unknown,
}

pub const UUID: &str = &include_str!(concat!(env!("OUT_DIR"), "/uuid.txt"));
EOF

# 在Cargo.toml 中添加依赖项
# sed -i '$a [build-dependencies]\n' ./Cargo.toml
sed -i '/\[dependencies\]/a num_enum = { version = "0.7", default-features = false }\n' ./Cargo.toml

# 修改Cargo.toml 中的版本和作者
sed -i "s/^authors = .*/authors = [\"$authors\"]/" ./Cargo.toml

cd ..

# 创建 ta 项目
mkdir -p ta/src
cd ta || exit 1
cargo init --bin
cat <<EOF > Makefile
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

UUID ?= \$(shell cat "../uuid.txt")

TARGET ?= aarch64-unknown-linux-gnu
OBJCOPY := objcopy

TA_SIGN_KEY ?= \$(TA_DEV_KIT_DIR)/keys/default_ta.pem
SIGN := \$(TA_DEV_KIT_DIR)/scripts/sign_encrypt.py
OUT_DIR := \$(CURDIR)/../target/\$(TARGET)/release

all: ta strip sign

ta:
	@cargo build --target \$(TARGET) --release --verbose

strip: ta
	@\$(OBJCOPY) --strip-unneeded \$(OUT_DIR)/ta \$(OUT_DIR)/stripped_ta

sign: strip
	@\$(SIGN) --uuid \$(UUID) --key \$(TA_SIGN_KEY) --in \$(OUT_DIR)/stripped_ta --out \$(OUT_DIR)/\$(UUID).ta
	@echo "SIGN =>  \${UUID}"

clean:
	@cargo clean

install: 
	sudo cp \$(OUT_DIR)/\$(UUID).ta /usr/lib/optee_armtz/\$(UUID).ta

EOF

cat <<EOF > ./src/main/rs
// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![no_std]
#![no_main]
#![feature(c_size_t)]

use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{Error, ErrorKind, Parameters, Result};
use proto::Command;

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session() {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    let mut values = unsafe { params.0.as_value().unwrap() };

    let cmd = Command::try_from(cmd_id).map_err(|e| {
        optee_utee::trace_println!("Unknown cmd {}, err is {}", cmd_id, e);
        ErrorKind::BadParameters
    })?;

    match cmd {
        Command::IncValue => {
            values.set_a(values.a() + 100);
            Ok(())
        }
        Command::DecValue => {
            values.set_a(values.a() - 100);
            Ok(())
        }
        _ => Err(Error::new(ErrorKind::BadParameters)),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
EOF

cat <<EOF > ./build.rs
// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use proto;
use optee_utee_build::{Error, RustEdition, TaConfig};

fn main() -> Result<(), Error> {
    let config = TaConfig::new_default_with_cargo_env(proto::UUID)?;

    optee_utee_build::build(RustEdition::Edition2024, config)

}
EOF

cat <<EOF > ./src/main.rs
// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![no_std]
#![no_main]
#![feature(c_size_t)]

use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{Error, ErrorKind, Parameters, Result};
use proto::Command;

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session() {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    let mut values = unsafe { params.0.as_value().unwrap() };

    let cmd = Command::try_from(cmd_id).map_err(|e| {
        optee_utee::trace_println!("Unknown cmd {}, err is {}", cmd_id, e);
        ErrorKind::BadParameters
    })?;

    match cmd {
        Command::IncValue => {
            values.set_a(values.a() + 100);
            Ok(())
        }
        Command::DecValue => {
            values.set_a(values.a() - 100);
            Ok(())
        }
        _ => Err(Error::new(ErrorKind::BadParameters)),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
EOF

# 在Cargo.toml 中添加依赖项
sed -i '/\[dependencies\]/a proto = { path = "../proto" } \noptee-utee = { git = "https://github.com/apache/incubator-teaclave-trustzone-sdk.git", branch = "main", default-features = false }\noptee-utee-sys = { git = "https://github.com/apache/incubator-teaclave-trustzone-sdk.git", branch = "main", default-features = false }\n' ./Cargo.toml

sed -i '$a [build-dependencies]\nproto = { path = "../proto" }\noptee-utee-build = "0.2.0"\n' ./Cargo.toml
sed -i '$a [profile.release]\npanic = "abort"\nlto = true\nopt-level = 1\n' ./Cargo.toml

# 修改Cargo.toml 中的版本和作者
sed -i "s/^authors = .*/authors = [\"$authors\"]/" ./Cargo.toml

cd ..

# 创建 uuid.txt
echo $(python -c "import uuid; print(uuid.uuid4())") > uuid.txt && truncate -s 36 uuid.txt

# 输出成功信息
echo "Rust TA project '$PROJECT_NAME' initialized successfully!"