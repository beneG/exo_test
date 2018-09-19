

### Summary

This program is an auction implementation using exonum

Features:

* create new user/wallet
* create product
* sell product on auction
* make bids

### Compile, deploy and run application

```sh
mkdir workdir
```

```sh
cd workdir
```

First of all you need to download exonum source code:

```sh
git clone https://github.com/exonum/exonum.git
```

Then we need to clone this appication source code:

```sh
git clone hhttps://github.com/beneG/exo_test.git
```

Then build application:

```sh
cd exo_test
```

```sh
cargo build
```


Generate blockchain configuration:

```sh
mkdir config
```

```sh
./target/debug/auction generate-template config/common.toml --validators-count 4
```

Generate templates of nodes configurations:

```sh
./target/debug/auction generate-config config/common.toml  config/pub_1.toml config/sec_1.toml --peer-address 127.0.0.1:6331

./target/debug/auction generate-config config/common.toml  config/pub_2.toml config/sec_2.toml --peer-address 127.0.0.1:6332

./target/debug/auction generate-config config/common.toml  config/pub_3.toml config/sec_3.toml --peer-address 127.0.0.1:6333

./target/debug/auction generate-config config/common.toml  config/pub_4.toml config/sec_4.toml --peer-address 127.0.0.1:6334
```

Finalize generation of nodes configurations:

```sh
./target/debug/auction finalize --public-api-address 0.0.0.0:8200 --private-api-address 0.0.0.0:8091 config/sec_1.toml config/node_1_cfg.toml --public-configs config/pub_1.toml config/pub_2.toml config/pub_3.toml config/pub_4.toml

./target/debug/auction finalize --public-api-address 0.0.0.0:8201 --private-api-address 0.0.0.0:8092 config/sec_2.toml config/node_2_cfg.toml --public-configs config/pub_1.toml config/pub_2.toml config/pub_3.toml config/pub_4.toml

./target/debug/auction finalize --public-api-address 0.0.0.0:8202 --private-api-address 0.0.0.0:8093 config/sec_3.toml config/node_3_cfg.toml --public-configs config/pub_1.toml config/pub_2.toml config/pub_3.toml config/pub_4.toml

./target/debug/auction finalize --public-api-address 0.0.0.0:8203 --private-api-address 0.0.0.0:8094 config/sec_4.toml config/node_4_cfg.toml --public-configs config/pub_1.toml config/pub_2.toml config/pub_3.toml config/pub_4.toml
```


Run nodes:

```sh
./target/debug/auction run --node-config config/node_1_cfg.toml --db-path config/db1 --public-api-address 0.0.0.0:8200

./target/debug/auction run --node-config config/node_2_cfg.toml --db-path config/db2 --public-api-address 0.0.0.0:8201

./target/debug/auction run --node-config config/node_3_cfg.toml --db-path config/db3 --public-api-address 0.0.0.0:8202

./target/debug/auction run --node-config config/node_4_cfg.toml --db-path config/db4 --public-api-address 0.0.0.0:8203
```


### RESTful API Routes

#### HTTP GET requests:

Get users list:

```127.0.0.1:8200/api/services/auction/v1/users```

Get user information:

```<host>:8200/api/services/auction/v1/user?pub_key=<users_public_key>```

Get products list:

```<host>:8200/api/services/auction/v1/products```

Get specific product information:

```<host>:8200/api/services/auction/v1/product?id=<product_id>```

Get users products list:

```<host>:8200/api/services/auction/v1/user/products?pub_key=<users_public_key>```

Get users auctions list:

```<host>:8200/api/services/auction/v1/user/auctions?pub_key=<users_public_key>```

Get bids on specific auction:

```<host>:8200/api/services/auction/v1/auction/bids?id=<auction_id>```

Get auction information (with bids):

```<host>:8200/api/services/auction/v1/auction?id=<auction_id>```

Get auctions list:

```<host>:8200/api/services/auction/v1/auctions```



#### HTTP POST requests (Transactions):

In order to send transaction to the system you have to make HTTP POST request to URL

```<host>:8200/api/services/auction/v1/transcactions```

Or in case of MakeBid transaction:

```<host>:8200/api/services/auction/v1/sync_transcactions```

All parameters are passed through data field in json format

`protocol_version` field must be `0`

`service_id` field must be `73`

`message_id` field determines which transaction will be executed:

* `0` create new user/wallet
* `1` create product
* `2` add money to user
* `3` create auction
* `4` make bid
* `5` close auction


Examples of each transaction type:

Create user accout transaction
```javascript
{
  "body": {
    "public_key": "fdddafffffffff434107ce52c287001c968a1b6eca3e5a1eb62a2419e2924b85",
    "name": "DavidBoue"
  },
  "protocol_version": 0,
  "service_id": 73,
  "message_id": 0,
  "signature":"f000faffdddf663775848b3db656bca685e085391e2b00b0e115679fd45443ef58a5abeb555ab3d5f7a3cd27955a2079e5fd486743f36515c8e5b"
}
```

Create product transaction
```javascript
{
  "body": {
    "public_key": "fdddafffffffff434107ce52c287001c968a1b6eca3e5a1eb62a2419e2924b85",
    "name": "Quantum computer"
  },
  "protocol_version": 0,
  "service_id": 73,
  "message_id": 1,
  "signature":"f000faffdddf663775848b3db656bca685e085391e2b00b0e115679fd45443ef58a5abeb555ab3d5f7a3cd27955a2079e5fd486743f36515c8e5b"
}
```

Issue
```javascript
{
  "body": {
    "public_key": "fdddafffffffff434107ce52c287001c968a1b6eca3e5a1eb62a2419e2924b85"
  },
  "protocol_version": 0,
  "service_id": 73,
  "message_id": 2,
  "signature":"f000faffdddf663775848b3db656bca685e085391e2b00b0e115679fd45443ef58a5abeb555ab3d5f7a3cd27955a2079e5fd486743f36515c8e5b"
}
```



Create auction transaction
```javascript
{
  "body": {
    "public_key": "fdddafffffffff434107ce52c287001c968a1b6eca3e5a1eb62a2419e2924b85",
    "product_id": "0000000000ffff434107ce52c287001c968a1b6eca3e5a1eb62a2419e2924b85",
    "start_price": 10
  },
  "protocol_version": 0,
  "service_id": 73,
  "message_id": 3,
  "signature":"f000faffdddf663775848b3db656bca685e085391e2b00b0e115679fd45443ef58a5abeb555ab3d5f7a3cd27955a2079e5fd486743f36515c8e5b"
}
```

**Make bid transaction**

In order to get block number and position in block this transaction must be submitted through slightly differrent URL

```127.0.0.1```

```javascript
{
  "body": {
    "public_key": "fdddafffffffff434107ce52c287001c968a1b6eca3e5a1eb62a2419e2924b85",
    "auction_id": 123,
    "value": 20
  },
  "protocol_version": 0,
  "service_id": 73,
  "message_id": 4,
  "signature":"f000faffdddf663775848b3db656bca685e085391e2b00b0e115679fd45443ef58a5abeb555ab3d5f7a3cd27955a2079e5fd486743f36515c8e5b"
}
```

Close auction transaction
```javascript
{
  "body": {
    "auction_id": 123
    "closing_party": "fdddafffffffff434107ce52c287001c968a1b6eca3e5a1eb62a2419e2924b85",
  },
  "protocol_version": 0,
  "service_id": 73,
  "message_id": 5,
  "signature":"f000faffdddf663775848b3db656bca685e085391e2b00b0e115679fd45443ef58a5abeb555ab3d5f7a3cd27955a2079e5fd486743f36515c8e5b"
}
```
