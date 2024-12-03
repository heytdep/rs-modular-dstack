# TODO: this is just a standard machine, we actually need to enable the tdx functionality for this whoe thing to make sense.
run_qemu() {
  echo "Starting QEMU VM..."
  qemu-system-x86_64 -D /tmp/qemu-guest.log \
    -accel kvm -m 16G -smp 4 \
    -name qemu-vm,process=qemu-vm,debug-threads=on -cpu host -nographic -nodefaults \
    -device virtio-net-pci,netdev=nic0 -netdev user,id=nic0,hostfwd=tcp::10022-:22,hostfwd=tcp::24070-:24070,hostfwd=tcp::24071-:24071 \
    -drive file=../flashbox/flashbox.raw,if=none,id=virtio-disk0 -device virtio-blk-pci,drive=virtio-disk0 \
    -bios /usr/share/ovmf/OVMF.fd \
    -chardev stdio,id=char0,mux=on,signal=off -mon chardev=char0 -serial chardev:char0 \
    -pidfile /tmp/qemu-pid.pid -machine q35 &
}
