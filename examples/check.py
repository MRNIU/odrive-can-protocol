# Copyright The odrive-can-protocol Contributors
# 在临时消费方中编译、测试适配模块，不向协议库加入驱动依赖。

import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description="在临时消费方中验证 CAN 帧适配模块")
    parser.add_argument("adapter", choices=["embedded_can", "embassy", "socketcan"])
    parser.add_argument("--target", help="仅交叉检查 no_std 适配模块")
    parser.add_argument("--chip", default="stm32h723vg", help="Embassy 芯片 feature")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    dependencies = 'embedded-can = "=0.4.1"\n'
    if args.adapter == "embedded_can":
        dependencies += 'bxcan = "=0.8.0"\n'
    elif args.adapter == "embassy":
        dependencies += (
            'embassy-stm32 = { version = "=0.6.0", features = ['
            + json.dumps(args.chip) + '] }\n'
        )
    else:
        if args.target:
            parser.error("SocketCAN 仅在 Linux 主机验证")
        dependencies += 'socketcan = { version = "=4.0.0", default-features = false }\n'
    with tempfile.TemporaryDirectory(prefix="odrive-adapter-") as temporary:
        directory = Path(temporary)
        adapter = root / "examples" / args.adapter
        (directory / "Cargo.toml").write_text(
            '[package]\nname = "adapter-check"\nversion = "0.0.0"\n'
            'edition = "2024"\npublish = false\n'
            '[lib]\npath = "lib.rs"\n'
            '[[test]]\nname = "frames"\npath = '
            + json.dumps(str(adapter / "tests.rs"))
            + '\n[dependencies]\nodrive-can-protocol = { path = '
            + json.dumps(str(root)) + ' }\n' + dependencies,
            encoding="utf-8",
        )
        (directory / "lib.rs").write_text(
            '#![no_std]\n#![forbid(unsafe_code)]\n#![deny(missing_docs)]\n'
            '//! 适配模块验证入口。\n#[path = '
            + json.dumps(str(adapter / "adapter.rs")) + ']\npub mod adapter;\n',
            encoding="utf-8",
        )
        environment = os.environ.copy()
        environment.setdefault("CARGO_TARGET_DIR", str(root / "target" / "adapters"))

        def cargo(*arguments):
            subprocess.run(["cargo", *arguments], cwd=directory, env=environment, check=True)

        cargo("fmt", "--check")
        if args.target:
            cargo("check", "--lib", "--target", args.target)
        else:
            cargo("test", "--all-targets")
            cargo("clippy", "--locked", "--all-targets", "--", "-D", "warnings")


if __name__ == "__main__":
    main()
