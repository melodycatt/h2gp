# HOW TO USE THE APP
download the pre-compiled app for your platform from the [releases](https://github.com/melodycatt/h2gp/releases) (you probably just want the latest one)
## how to know which one to download
### mac:
click the apple icon in the top left, click about this mac:

<img width="287" height="490" alt="Screenshot 2026-06-04 at 8 43 45 am" src="https://github.com/user-attachments/assets/c08e8ded-c89e-42b1-becb-e4cdb52f57a0" />

- if the text i highlighted says "Apple M_" then you should download the arm64-apple one
- if it says intel or basically anything other than the first option, its probably the x86_64-apple one. if it doesnt work try the arm64-apple one

### windows:
go to `Settings > System > About` and look at System Type:

<img width="750" height="576" alt="image" src="https://github.com/user-attachments/assets/99e24e38-5463-4714-b531-383ddb48d151" />
- if it says x86 or x64 in the highlighted text, download the x86_64-windows one
- if it says ARM64 or ARM or AARCH64 or something similar to those then download the arm64-windows one

if youre on linux why is your school computer on linux. idiot

## how to run it
all versions come with the actual app and `config.toml`. config.toml contains some minimal config that you probably dont need to touch. ill get to that later
### mac:
open the dmg file and drag the app out of it, probably into the Applications folder (idk if that makes a difference)
macOS has some really stingy security rules so youll probably get a popup saying that it cant verify the app isnt malware. if you do, theres two ways to fix it:
1. go to settings > security & privacy, scroll to the bottom and you should see this:\
  <img width="476" height="180" alt="Screenshot 2026-06-23 at 9 48 37 am" src="https://github.com/user-attachments/assets/83200a52-3f01-41bd-ac1c-3e11d9047632" />\
  click open anyway. itll probably ask for your password or fingerprint or whatever
2. open terminal (Cmd+Space then type terminal) and type this into it:
   ```
   xattr -rd com.apple.quarantine
   ```
   then drag the folder you downloaded (make sure its unzipped) into the terminal. make sure there is a space between "com.apple.quarantine" and the path that gets copied into the terminal\
   it should look something like `xattr -rd com.apple.quarantine /Users/<your name>/Applications/H2GP.app/`. now press enter and this should remove the restrictions on the app.

currently, the app wont open if the receiver isnt plugged in. by default, it expects it to be plugged into **usb-c port 2** (~~the one closest to you~~ turns out, this changes between mac versions, meaning it could be either of the ports! its best to just try both). you can change this by changing the config.toml value to `port = 1`, meaning it should be plugged into usb-c port 1
### windows:
windows *also* has stingy security rules, except i dont have a windows laptop so i dont know how to get around them! basically just look it up or ask chatgpt if you cant download or run the app

**it will not work out of the box**. i will write a prompt for chatgpt to figure out how to get it to work if i remember. if not just send it the contents of `config.toml` and ask it how to find the equivalent of `/dev/tty.usbmodem101` for your windows laptop for an esp32 jtag serial and what to replace port with to get it to work (tell it it should be a string path)

## how to use it
the receiver will not be connected to the brain on startup. just click the connect button on the commands panel. by default this connects to brain 3, which you can change in the config.toml file with `default_addr`. while the app is running you can switch to a different brain with the "brain #" input, then click the connect button

the brains have labels on them for numbers. registering a new brain is annoying but i can help if thats needed.

the brain then needs to be connected to the receiver. in the commands panel, there are two receiver numbers, because the brain can send to to receivers at once. if a receiver channel is set to 0, then the brain wont send anything for it, otherwise it should be set to the number on the label of the receiver. if youre only using one receiver, you should set one channel to 0. you have to click send command to apply this

the send wait checkbox tells the brain whether to wait for the receivers to receive the data before continuing. this should usually be enabled, but if youre getting lag issues disabling this may help. it could cause other issues like packet loss though, idk. best to leave it on.


every floating window/panel can be dragged around

the graphs show:
1. the voltage of the battery and fuel cell. you can hover over them to see which is which
2. the current of the battery and fuel cell. they should be vaguely opposite, with the battery fluctuating as you drive. hover to see which is which
3. temperature values:
  a) ~~thermistor readings. this is in millivolts it think so it will be in the thousands. and i dont know how to convert it to degrees so its mostly useless. dont really worry about it~~ removed
  b) brain temperature in ºC. just as useless because who cares about the brain getting hot. unless its REALLY hot. ~~because the thermistor is in the thousands the graph will be scalled so you can barely tell what the temperature is. just hover over the line~~

there are checkboxes on the logo window thing:
- motion shows the acceleration and gyro data, mostly useless
- log shows output of the receiver, as well as raw data. THIS IS VERY USEFUL ill get to it later
- long term graphs: the app takes every 50th reading, and stores them. when long term graphs is enabled it shows these so you can see a general trend of the data
- commands shows the command panel. the options should be self explanatory

### log
this will show raw data, which can be useful for trouble shooting. you can also see a regular 'receiver alive' message to be sure its still alive. if the receiver crashes, youll see a `PANIC:` message in the log. if you know how to code you can fix it and make a pull request, if you dont make an issue on the issues tab at the top of this page and paste the panic message. i might get to it if i see it.

if the brain crashes you *might* see a different kind of panic message but idk its weird. just restart the brain if you see a panic message but the receivers still alive i think
if the brain stops responding to messages and sending data but its still alive, check if the 'free_heap' value in the raw data is less than like, 10,000. if it is just restart everything. probably restart everything anyway.

## troubleshooting
if the app stops responding, buttons stop working, you cant restart the receiver or something like that:
1. close the app
2. unplug receiver
3. switch the brain off power and turn it back on again
4. wait until the blue light on the brain turns on
5. plug the receiver in again
6. restart app




# a short guide to the repository
### anything starting with FCCU
this is all billy harris-nevill's old code i copied off of. it was terrible and made with chatgpt. be glad you dont have to deal with it
### random letters and numbers
these are drivers for different ICs on the board that i ported to rust. you can ignore them really, and i didn't bother to document them very well :b

for your interest:
- ina226 = voltage and current sensor
- icm42688 = motion sensor (accel + gyro)
- pca9685 = servo driver from another of billy's boards. i could never get the sbus working so we didnt end up using it. it was for steering stabilisation - maybe something to work on if interested but mostly useless anyway
### brain
code for the brain. not commented at all, messy, slapped together with masking tape and prayers
### receiver
code for the receiver. not commented at all, but slightly less messy because its easier to debug when its plugged into my laptop
### record_mac
a thing for compilation that registers new brains so we can connect to them easily
### interface
the code for the app. really really unreadable i dont care though.
### *_macs.txt
registered mac-addresses for the esp32s. only used at compilation time, i would suggest NOT editing this at all to be safe
### INA226_WE.*
a driver i ported to rust
### kitty.session
a thing only for me really, it just makes setting up the workspace easier on the kitty terminal
### merge.sh
for merging the repos into this one big one, idk why im keeping it
### run.sh
helper script for compilation + record_mac

