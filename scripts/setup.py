#!/usr/bin/python
#
# Copyright (c) 2018 University of Utah
#
# Permission to use, copy, modify, and distribute this software for any
# purpose with or without fee is hereby granted, provided that the above
# copyright notice and this permission notice appear in all copies.
#
# THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR(S) DISCLAIM ALL WARRANTIES
# WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
# MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL AUTHORS BE LIABLE FOR
# ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
# WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
# ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
# OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

import os
import sys
import argparse
import subprocess

"""Dictionary of different colors that can be printed to the screen.
"""
colors = {
    "bold": '\033[1m',
    "end" : '\033[0m',
}

"""This function prints a passed in string in the specified color.
"""
def printColor(color, string):
    print colors[color] + string + colors["end"]

"""This function fixes dependencies inside the db and ext/test crates.
"""
def setupCargo():
    printColor("bold", "=============== Fixing Deps ==========================")
    fix = "cargo generate-lockfile; " + \
          "cargo update -p spin:0.4.10 --precise 0.4.7; " + \
          "cargo update -p serde:1.0.94 --precise 1.0.67; " + \
          "cargo update -p serde_derive:1.0.94 --precise 1.0.67; " + \
          "cargo update -p env_logger:0.5.13 --precise 0.5.3; " + \
          "cargo update -p rustc-demangle:0.1.15 --precise 0.1.13; " + \
          "cargo update -p twox-hash:1.4.2 --precise 1.1.1; " + \
          "cargo update -p backtrace:0.3.32 --precise 0.3.20; " + \
          "cargo update -p backtrace-sys:0.1.30 --precise 0.1.28; " + \
          "cargo update -p atty:0.2.12 --precise 0.2.11; "

    # Fix dependencies inside db.
    cmd = "cd db; " + fix + "cd ../"
    subprocess.check_call(cmd, shell=True)

    # Fix dependencies inside ext/test.
    cmd = "cd ext/test; " + fix + "cd ../../"
    subprocess.check_call(cmd, shell=True)

    # Fix dependencies inside ext/pushback.
    cmd = "cd ext/pushback; " + fix + "cd ../../"
    subprocess.check_call(cmd, shell=True)


    # Fix dependencies inside splinter.
    cmd = "cd splinter; " + fix + "cd ../"
    subprocess.check_call(cmd, shell=True)

"""This function first compiles DPDK using Netbricks scripts on CloudLab's xl170.
"""
def setupDpdk():
    printColor("bold", "=============== Compiling DPDK =======================")
    subprocess.check_call("./net/3rdparty/get-dpdk.sh", shell=True)

    print ""
    printColor("bold", "=============== Binding NIC to DPDK ==================")
    # First, find the PCI-ID of the first active 10 GigE NIC.
    cmd = "./net/3rdparty/dpdk/usertools/dpdk-devbind.py --status-dev=net |" + \
            " grep ens1f1 | grep Active | tail -1 | awk '{ print $1 }'"
    pci = subprocess.check_output(cmd, shell=True)

    # Print out the PCI and MAC address of the NIC.
    cmd = "ls /sys/bus/pci/devices/" + str(pci).rstrip() + "/net/"
    net = subprocess.check_output(cmd, shell=True)
    cmd = "ethtool -P " + str(net).rstrip() + " | awk '{ print $3 }'"
    mac = subprocess.check_output(cmd, shell=True)
    printColor("bold", "NIC PCI ADDRESS: " + str(pci).rstrip())
    printColor("bold", "NIC MAC ADDRESS: " + str(mac).rstrip())

    # Write out the PCI and MAC addresses to a file.
    subprocess.check_output("rm -Rf ./nic_info", shell=True)
    subprocess.check_output("echo \"pci: " + str(pci).rstrip() + \
                            "\" >> ./nic_info", shell=True)
    subprocess.check_output("echo \"mac: " + str(mac).rstrip() + \
                            "\" >> ./nic_info", shell=True)

    return

"""This function sets up the vim editor.
"""
def setupDevEnvt():
    printColor("bold", "=============== Setting up Dev Environment ===========")
    subprocess.check_call("cp ./misc/dev/vimrc-sample ~/.vimrc", shell=True)
    subprocess.check_call("cp -r ./misc/dev/vim ~/.vim", shell=True)
    subprocess.check_call("vim +PlugClean +PlugInstall +qall", shell=True)

"""This function installs the nightly version of Rust.
"""
def installRust():
    printColor("bold", "=============== Installing Rust ======================")
    subprocess.check_call("curl -s https://sh.rustup.rs -sSf | " +\
                          "sh -s -- --default-toolchain nightly-2018-08-02 -y",
                          shell=True)
    os.environ["PATH"] += ":" + os.environ["HOME"] + "/.cargo/bin"
    return

def setupVScode():
    printColor("bold","================ Installing IDE ===================")
    subprocess.check_call("sudo apt update",  shell=True)
    subprocess.check_call("sudo apt -y install libnotify4 libnspr4 libnss3 libnss3-nssdb", shell=True)
    subprocess.check_call("sudo apt -y install libsecret-1-0 libsecret-common libxkbfile1", shell=True)
    subprocess.check_call("sudo apt -y install notification-daemon gitk git-gui", shell=True)
    subprocess.check_call("wget https://az764295.vo.msecnd.net/stable/61122f88f0bf01e2ac16bdb9e1bc4571755f5bd8/code_1.30.2-1546901646_amd64.deb",
                           shell=True)
    subprocess.check_call("sudo dpkg -i code_1.30.2-1546901646_amd64.deb", shell=True)
    subprocess.check_call("rm  code_1.30.2-1546901646_amd64.deb", shell=True)
    return

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=\
                                     'Setup a machine for Sandstorm')
    parser.add_argument('--full', action='store_true',
                        help='Performs a full setup on the box.')
    parser.add_argument('--installRust', action='store_true',
                        help='Installs rust.')
    parser.add_argument('--setupDevEnv', action='store_true',
                        help='Sets up development tools (vim etc).')
    parser.add_argument('--installDpdk', action='store_true',
                        help='Builds and installs DPDK.')
    parser.add_argument('--fixCargoDep', action='store_true',
                        help='Fixes all cargo dependencies.')
    parser.add_argument('--installIDE', action='store_true',
                        help='install VS code and git-gui.')
    args = parser.parse_args()

    # First, install Rust.
    if args.full or args.installRust:
        installRust()

    # Then, setup the development environment.
    if args.full or args.setupDevEnv:
        setupDevEnvt()

    # Next, setup DPDK.
    if args.full or args.installDpdk:
        setupDpdk()

    # Finally, fix dependencies.
    if args.full or args.fixCargoDep:
        setupCargo()

    if args.full or args.installIDE:
        setupVScode()

    print "\n\tRun- source $HOME/.cargo/env\n"
    sys.exit(0)
