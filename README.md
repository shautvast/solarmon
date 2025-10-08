# solarmon
* charts solaredge inverter data using its REST api like their app also does
* sends alerts if around 12:00 no output is measured. Mine had died without a word last summer...
* alerting on your phone using pushover
* solaredge api has at least 15 mins resolution. Its response is cached to prevent overloading their server.

**start**
* clone this repo
* create .env file that contains
  * SOLAREDGE_API_KEY
  * SOLAREDGE_SITE_ID
  * PUSHOVER_USER_ID
  * PUSHOVER_API_KEY
  * BIND_ADDR eg. 0.0.0.0:3000
  * CALL_HOME url to include in the pushover notification
* cargo run

After successful startup a informational message is sent to pushover.