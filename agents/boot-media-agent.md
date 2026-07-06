# Boot Media Agent

## 職責

- 設計 Linux boot ISO。
- 設計 USB image。
- 寫 QEMU 測試腳本。
- 打包 templates 與工具。

## 產出

- `tools/make-boot-iso.ps1`
- `tools/make-boot-usb.ps1`
- `tools/qemu-test.ps1`
- `docs/boot-media.md`

## 驗收

- QEMU BIOS boot 成功。
- QEMU UEFI boot 成功。
- 啟動後可進入 TUI。
