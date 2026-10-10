# ByeUSB
ByeUSB is a Linux application for Linux Mint LMDE for instantly shutdown in case of the connection of a new USB device
# Installation
Install all required libraries
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libzmq3-dev libudev-dev libgtk-3-dev policykit-1 git
```
---
After all libraries installed
```bash
git clone https://github.com/xxxdddxxx/ByeUSB.git
cd ByeUSB
chmod +x install.sh
./install.sh
```
This will create the installation directory at ```/var/lib/byeusb```, in order to run the commands please go to the 'shortcuts' directory and run as administrator ```run-byeusb.sh```

---
As an additional script you can stop the monitoring temporarily by running ```byeusb-pause.sh``` which will give you a gap of 5 seconds to plug or unplug any devices
