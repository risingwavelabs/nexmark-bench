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
  p_time TIMESTAMPTZ as proctime()
) WITH (
  connector = 'kafka',
  topic = 'nexmark-events',
  properties.bootstrap.server = '${KAFKA_HOST}:${KAFKA_PORT}',
  scan.startup.mode = 'earliest'
) ROW FORMAT JSON;
