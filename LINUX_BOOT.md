# Booting Arch Linux from DBFS (in a VM)

The following instructions should not take more than an hour on recent-ish hardware (takes a bit over 35 minutes on a Ryzen 5 3500U machine).

## Step 0: Init

```
sudo pacman -S arch-install-scripts mariadb
mkdir dbramfs && cd dbramfs
wget https://github.com/clearlinux/common/raw/refs/heads/master/OVMF.fd
```

## Step 1: Install Arch inside tmpfs

```
mkdir tmp
sudo mount -t tmpfs -o size=100% none tmp
sudo pacstrap -K tmp base linux vim neofetch
sudo arch-chroot tmp
```

Inside the chroot:

```
ln -sf /usr/share/zoneinfo/Europe/Prague /etc/localtime
sed -i "s/#en_US.UTF-8/en_US.UTF-8/g" /etc/locale.gen
locale-gen
echo "dbfsvm" > /etc/hostname
echo -n "1234" | passwd -s
exit
```

## Step 2: Build `dbfs` for the initial install
```
git clone git@gordon.zumepro.cz:tul/dbfs
cd dbfs
cargo build
```

## Step 3: Prepare MariaDB

```
sudo systemctl start mariadb
sudo mariadb-install-db --user=mysql --basedir=/usr --datadir=/var/lib/mysql
sudo mariadb -e "DROP DATABASE \`dbfs\`;"
user_creation="CREATE DATABASE \`dbfs\`; GRANT ALL PRIVILEGES ON \`dbfs\`.* TO 'dbfs'@'localhost' IDENTIFIED BY 'dbfs'; USE \`dbfs\`;"
data_file=$(cat "./sql/mysql/dbfs.sql")
sudo mariadb -e "$user_creation$data_file" 
```

## Step 4: Import the tmpfs contents into dbfs

```
sudo ./target/debug/dbfs import ../tmp
sudo umount -l ../tmp
```

## Step 5: Build `dbfs` for the target

```
sed -i "s/127.0.0.1/10.0.2.2/g" src/settings.rs
just build
```

## Step 6: Prepare `dracut`

```
cd ..
git clone https://github.com/dracutdevs/dracut
cd dracut
git checkout 5d2bda46f4e75e85445ee4d3bd3f68bf966287b9
cp ../dbfs/target/release/dbfs .
```

## Step 7: Build `dracut`

Start an Arch Linux container:

```
podman run -it --name dbfs_initramfs -v .:/dracut docker.io/archlinux:latest bash
```

## Step 7.1: Initramfs generation inside the container
 
```
sed -i "s/NoProgressBar/ILoveCandy/g" /etc/pacman.conf
sed -i "s/#Color/Color/g" /etc/pacman.conf
pacman -Sy --noconfirm base-devel linux fuse3 fuseiso cdrtools asciidoc dhclient cpio wget vim
cd dracut
./configure
make
mkdir -p modules.d/90fuse
mkdir -p efi_firmware/EFI/BOOT
cp dbfs /usr/bin
echo '#!'"/bin/bash
check() {
    require_binaries dbfs chroot ps df vim wget fusermount fuseiso mkisofs || return 1
    return 0
}

depends() {
    return 0
}

install() {
    inst_multiple dbfs chroot ps df vim wget fusermount fuseiso mkisofs
    return 0
}" > modules.d/90fuse/module-setup.sh
# Following patch is escaped for use with sed, percentage sign is used as command delimiter
patch="echo \">>> Initializing subsystems for dbfs...\"
modprobe fuse
modprobe e1000
ip link set lo up
ip link set eth0 up
dhclient eth0
ip route add default via 10.0.2.2 dev eth0 proto dhcp src 10.0.2.15
mkdir /dbfs
echo \">>> Starting dbfs...\"
dbfs mount --allow-other /dbfs \&
sleep 1
echo \">>> Mounting VFS...\"
mount --rbind /sys /dbfs/sys
mount --rbind /dev /dbfs/dev
mount -t proc /proc /dbfs/proc
echo \">>> Done.\"
exec chroot /dbfs /sbin/init
"
sed -i "s/^\[ -z \"\$root.*//g" modules.d/99base/init.sh
sed -i "s%make_trace_mem \"hook initqueue\"%${patch//$'\n'/\\n}\nmake_trace_mem \"hook initqueue\"%g" modules.d/99base/init.sh
./dracut.sh --kver `ls -1 /lib/modules | tail -n 1` --uefi efi_firmware/EFI/BOOT/BOOTX64.efi --force -l -N --no-hostonly-cmdline --modules "base bash fuse shutdown network" --add-drivers "target_core_mod target_core_file e1000" --kernel-cmdline "ip=dhcp rd.shell=1"
exit
```

## Step 8: Boot

In theory, the system should now be able to boot up in a few minutes. Log in as "root" with password "1234".

```
qemu-system-x86_64 -bios ../OVMF.fd -m 4G -drive format=raw,file=fat:rw:./efi_firmware -netdev user,id=network0 -device e1000,netdev=network0
```

