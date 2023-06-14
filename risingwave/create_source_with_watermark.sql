CREATE SOURCE nexmark (
  event_type BIGINT,
  person STRUCT<"id" BIGINT,
                "name" VARCHAR,
                "email_address" VARCHAR,
                "credit_card" VARCHAR,
                "city" VARCHAR,
                "state" VARCHAR,
                "date_time" TIMESTAMP,
                "extra" VARCHAR>,
  auction STRUCT<"id" BIGINT,
                "item_name" VARCHAR,
                "description" VARCHAR,
                "initial_bid" BIGINT,
                "reserve" BIGINT,
                "date_time" TIMESTAMP,
                "expires" TIMESTAMP,
                "seller" BIGINT,
                "category" BIGINT,
                "extra" VARCHAR>,
  bid STRUCT<"auction" BIGINT,
            "bidder" BIGINT,
            "price" BIGINT,
            "channel" VARCHAR,
            "url" VARCHAR,
            "date_time" TIMESTAMP,
            "extra" VARCHAR>, 
  date_time TIMESTAMP AS
            CASE
                WHEN event_type = 0 THEN (person).date_time
                WHEN event_type = 1 THEN (auction).date_time
                ELSE (bid).date_time
            END,
  WATERMARK FOR date_time AS date_time - INTERVAL '4' SECOND
) WITH (
  connector = 'kafka',
  topic = 'nexmark-events',
  properties.bootstrap.server = '${KAFKA_HOST}:${KAFKA_PORT}',
  scan.startup.mode = 'earliest'
) ROW FORMAT JSON;
