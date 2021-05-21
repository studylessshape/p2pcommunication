## Protocol introduction
### Info
This is a small program that I use to practice my ability on rust.

It was p2p communication ago. But now it is room-to-people.

Of course, it is so hard to use. The reason is I can't use multicast to send the bytes of message and recevie message. That's why it only can input ip to join room under same WLAN.

### Use
Clone code to your directory by command below.
```sh
git clone https://gitee.com/study_less_shape/p2pcommunication.git
```

Or clone by
```sh
git clone https://github.com/studylessshape/p2pcommunication.git
```

Then run by `cargo run`.

Consider safe, I don't recommand to use `cargo install` to run this program.

### Message Protocol Info
**Define**
```
[protocol_name;4][id;12][code;1][message;_]
```
**Name**: `MOYU`
**Code**
|code|command|
|----|-------|
|`0`|search message|
|`1`|connect request|
|`2`|connect reply|
|`3`|send or receive message|

### Notice
If you want to know the base logic, this [**code**](https://gitee.com/study_less_shape/p2pcommunication/blob/ff9b187a16905669e8d24199d99edb615a8d9606/src/main.rs) is your wish and you can copy it to use.