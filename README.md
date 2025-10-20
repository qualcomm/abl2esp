# abl2esp

*abl2esp* is a minimal reimplementation of ABL that will search for
*EFI\boot\bootaa64.efi* across all available file systems and attempt to load
and start what it finds.

**Disclaimer:** this tool is made available for developer convenience. It's not
intended for product usage.

## Building

Use *rustup* to install the aarch64-unknown-uefi target. Then build using:

```
cargo build --target aarch64-unknown-uefi
```

## Packaging

The *ABL* is a standard EFI PE32+ application, wrapped in a FV, inside a LZMA,
inside a FV, inside an ELF file, with authentications segments added, which is
stored in the *abl_a* and *abl_b* partitions.

To package our newly built *abl2esp* binary, setup EDK2 for the packaging:

```
# The edk source tree is huge, a shallow clone will do
git clone --recursive --depth=1 --shallow-submodules https://github.com/tianocore/edk2.git
cd edk2
make -C BaseTools
. ./edksetup.sh
cd .. # back to the abl2esp source tree
```

Then in the same shell run:
```
./package.sh
```

This should create *abl-unsigned.elf*, sign this with a test signature using
*sectools*, or equivalent tool, to generate the *abl.elf* to be loaded onto
your development board..

## Deployment

Flash **abl.elf** into **abl_a** and **abl_b** partitions.

## Contribute

With the goal of providing a convenient development environment for upstream
work, please do contribute to both implementation and documentation by opening
a Pull Request. Issues can be used to track issues with the implementation,
documentation, and device-specific issues.

See [CONTRIBUTING](CONTRIBUTING.md) for more information.

## License

Licensed under [BSD 3-Clause Clear License](LICENSE.txt).
