* charts solaredge inverter data using its REST api like their app also does
* sends alerts if at 12:00 no output is measured
* alerting using pushover

**start**
one this repo
create .env file that contains
SOLAREDGE_API_KEY
SOLAREDGE_SITE_ID
PUSHOVER_USER_ID
PUSHOVER_API_KEY
BIND_ADDR eg. 0.0.0.0:3000
CALL_HOME optional url to include in the pushover notification
cargo run
