CREATE SINK nexmark_q0
    AS
    SELECT auction, bidder, price, date_time
    FROM bid
    WITH ( connector = 'blackhole', type = 'append-only');

CREATE SINK nexmark_q1
    AS
    SELECT auction,
           bidder,
           0.908 * price as price,
           date_time
    FROM bid
    WITH ( connector = 'blackhole', type = 'append-only');


CREATE SINK nexmark_q2
    AS
    SELECT auction, price
    FROM bid
    WHERE auction = 1007
       OR auction = 1020
       OR auction = 2001
       OR auction = 2019
       OR auction = 2087
    WITH ( connector = 'blackhole', type = 'append-only');


CREATE SINK nexmark_q3
    AS
    SELECT P.name,
           P.city,
           P.state,
           A.id
    FROM auction AS A
             INNER JOIN person AS P on A.seller = P.id
    WHERE A.category = 10
      and (P.state = 'or' OR P.state = 'id' OR P.state = 'ca') 
    WITH ( connector = 'blackhole', type = 'append-only');


CREATE SINK nexmark_q4
    AS
    SELECT Q.category,
           AVG(Q.final) as avg
    FROM (SELECT MAX(B.price) AS final,
                 A.category
          FROM auction A,
               bid B
          WHERE A.id = B.auction
            AND B.date_time BETWEEN A.date_time AND A.expires
          GROUP BY A.id, A.category) Q
    GROUP BY Q.category
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q5
    AS
    SELECT
        AuctionBids.auction, AuctionBids.num
    FROM (
        SELECT
            bid.auction,
            count(*) AS num,
            window_start AS starttime
        FROM
            HOP(bid, date_time, INTERVAL '2' SECOND, INTERVAL '10' SECOND)
        GROUP BY
            bid.auction,
            window_start
    ) AS AuctionBids
    JOIN (
    	SELECT
            max(CountBids.num) AS maxn,
            CountBids.starttime_c
    	FROM (
    		SELECT
                count(*) AS num,
                window_start AS starttime_c
    		FROM
                HOP(bid, date_time, INTERVAL '2' SECOND, INTERVAL '10' SECOND)
            GROUP BY
                bid.auction,
                window_start
    		) AS CountBids
    	GROUP BY
            CountBids.starttime_c
    	) AS MaxBids
    ON
        AuctionBids.starttime = MaxBids.starttime_c AND
        AuctionBids.num >= MaxBids.maxn
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q7
    AS
    SELECT B.auction,
           B.price,
           B.bidder,
           B.date_time
    from bid B
             JOIN (SELECT MAX(price) AS maxprice,
                          window_end as date_time
                   FROM
                       TUMBLE(bid, date_time, INTERVAL '10' SECOND)
                   GROUP BY window_end) B1 ON B.price = B1.maxprice
    WHERE B.date_time BETWEEN B1.date_time - INTERVAL '10' SECOND
              AND B1.date_time
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');

CREATE SINK nexmark_q8
    AS
    SELECT P.id,
           P.name,
           P.starttime
    FROM (SELECT id,
                 name,
                 window_start AS starttime,
                 window_end   AS endtime
          FROM
              TUMBLE(person, date_time, INTERVAL '10' SECOND)
          GROUP BY id,
                   name,
                   window_start,
                   window_end) P
             JOIN (SELECT seller,
                          window_start AS starttime,
                          window_end   AS endtime
                   FROM
                       TUMBLE(auction, date_time, INTERVAL '10' SECOND)
                   GROUP BY seller,
                            window_start,
                            window_end) A ON P.id = A.seller
        AND P.starttime = A.starttime
        AND P.endtime = A.endtime
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q9
    AS
    SELECT id,
           item_name,
           description,
           initial_bid,
           reserve,
           date_time,
           expires,
           seller,
           category,
           auction,
           bidder,
           price,
           bid_date_time
    FROM (SELECT A.*,
                 B.auction,
                 B.bidder,
                 B.price,
                 B.date_time                                                                  AS bid_date_time,
                 ROW_NUMBER() OVER (PARTITION BY A.id ORDER BY B.price DESC, B.date_time ASC) AS rownum
          FROM auction A,
               bid B
          WHERE A.id = B.auction
            AND B.date_time BETWEEN A.date_time AND A.expires) tmp
    WHERE rownum <= 1
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q10 AS
    SELECT auction,
           bidder,
           price,
           date_time,
           TO_CHAR(date_time, 'YYYY-MM-DD') as date,
           TO_CHAR(date_time, 'HH:MI')      as time
    FROM bid
    WITH ( connector = 'blackhole', type = 'append-only');


CREATE SINK nexmark_q12 AS
    SELECT bidder,
           count(*) as bid_count,
           window_start,
           window_end
    FROM
           TUMBLE(bid, p_time, INTERVAL '10' SECOND)
    GROUP BY 
           bidder,
           window_start,
           window_end
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q14 AS
    SELECT auction,
           bidder,
           0.908 * price as price,
           CASE
               WHEN
                           extract(hour from date_time) >= 8 AND
                           extract(hour from date_time) <= 18
                   THEN 'dayTime'
               WHEN
                           extract(hour from date_time) <= 6 OR
                           extract(hour from date_time) >= 20
                   THEN 'nightTime'
               ELSE 'otherTime'
               END       AS bidTimeType,
           date_time
           -- extra
           -- count_char(extra, 'c') AS c_counts
    FROM bid
    WHERE 0.908 * price > 1000000
      AND 0.908 * price < 50000000
    WITH ( connector = 'blackhole', type = 'append-only');


CREATE SINK nexmark_q15 AS
    SELECT to_char(date_time, 'YYYY-MM-DD')                                          as "day",
           count(*)                                                                  AS total_bids,
           count(*) filter (where price < 10000)                                     AS rank1_bids,
           count(*) filter (where price >= 10000 and price < 1000000)                AS rank2_bids,
           count(*) filter (where price >= 1000000)                                  AS rank3_bids,
           count(distinct bidder)                                                    AS total_bidders,
           count(distinct bidder) filter (where price < 10000)                       AS rank1_bidders,
           count(distinct bidder) filter (where price >= 10000 and price < 1000000)  AS rank2_bidders,
           count(distinct bidder) filter (where price >= 1000000)                    AS rank3_bidders,
           count(distinct auction)                                                   AS total_auctions,
           count(distinct auction) filter (where price < 10000)                      AS rank1_auctions,
           count(distinct auction) filter (where price >= 10000 and price < 1000000) AS rank2_auctions,
           count(distinct auction) filter (where price >= 1000000)                   AS rank3_auctions
    FROM bid
    GROUP BY to_char(date_time, 'YYYY-MM-DD')
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q16 AS
    SELECT channel,
           to_char(date_time, 'YYYY-MM-DD')                                          as "day",
           max(to_char(date_time, 'HH:mm'))                                          as "minute",
           count(*)                                                                  AS total_bids,
           count(*) filter (where price < 10000)                                     AS rank1_bids,
           count(*) filter (where price >= 10000 and price < 1000000)                AS rank2_bids,
           count(*) filter (where price >= 1000000)                                  AS rank3_bids,
           count(distinct bidder)                                                    AS total_bidders,
           count(distinct bidder) filter (where price < 10000)                       AS rank1_bidders,
           count(distinct bidder) filter (where price >= 10000 and price < 1000000)  AS rank2_bidders,
           count(distinct bidder) filter (where price >= 1000000)                    AS rank3_bidders,
           count(distinct auction)                                                   AS total_auctions,
           count(distinct auction) filter (where price < 10000)                      AS rank1_auctions,
           count(distinct auction) filter (where price >= 10000 and price < 1000000) AS rank2_auctions,
           count(distinct auction) filter (where price >= 1000000)                   AS rank3_auctions
    FROM bid
    GROUP BY to_char(date_time, 'YYYY-MM-DD'), channel 
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q17 AS
    SELECT auction,
           to_char(date_time, 'YYYY-MM-DD')                           AS day,
           count(*)                                                   AS total_bids,
           count(*) filter (where price < 10000)                      AS rank1_bids,
           count(*) filter (where price >= 10000 and price < 1000000) AS rank2_bids,
           count(*) filter (where price >= 1000000)                   AS rank3_bids,
           min(price)                                                 AS min_price,
           max(price)                                                 AS max_price,
           avg(price)                                                 AS avg_price,
           sum(price)                                                 AS sum_price
    FROM bid
    GROUP BY to_char(date_time, 'YYYY-MM-DD'), auction 
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q18 AS
    SELECT auction, bidder, price, channel, url, date_time
    FROM (SELECT *,
                 ROW_NUMBER() OVER (
                     PARTITION BY bidder, auction
                     ORDER BY date_time DESC
                     ) AS rank_number
          FROM bid)
    WHERE rank_number <= 1
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');

CREATE SINK nexmark_q19 AS
    SELECT auction, bidder, price, channel, url, date_time, extra
    FROM (SELECT *,
                 ROW_NUMBER() OVER (
                     PARTITION BY auction
                     ORDER BY price DESC
                 ) AS rank_number
          FROM bid)
    WHERE rank_number <= 10
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');

CREATE SINK nexmark_q20 AS
    SELECT auction,
           bidder,
           price,
           channel,
           url,
           B.date_time as bid_date_time,
           B.extra     as bid_extra,
           item_name,
           description,
           initial_bid,
           reserve,
           A.date_time as auction_date_time,
           expires,
           seller,
           category,
           A.extra     as auction_extra
    FROM bid AS B
             INNER JOIN auction AS A on B.auction = A.id
    WHERE A.category = 10
    WITH ( connector = 'blackhole', type = 'append-only');


CREATE SINK nexmark_q21 AS
    SELECT auction,
           bidder,
           price,
           channel,
           CASE
               WHEN LOWER(channel) = 'apple' THEN '0'
               WHEN LOWER(channel) = 'google' THEN '1'
               WHEN LOWER(channel) = 'facebook' THEN '2'
               WHEN LOWER(channel) = 'baidu' THEN '3'
               ELSE (regexp_match(url, '(&|^)channel_id=([^&]*)'))[2]
               END
               AS channel_id
    FROM bid
    WHERE (regexp_match(url, '(&|^)channel_id=([^&]*)'))[2] is not null
       or LOWER(channel) in ('apple', 'google', 'facebook', 'baidu')
    WITH ( connector = 'blackhole', type = 'append-only');


CREATE SINK nexmark_q22 AS
    SELECT auction,
           bidder,
           price,
           channel,
           split_part(url, '/', 4) as dir1,
           split_part(url, '/', 5) as dir2,
           split_part(url, '/', 6) as dir3
    FROM bid
    WITH ( connector = 'blackhole', type = 'append-only');


CREATE SINK nexmark_q101 AS
    SELECT
        a.id AS auction_id,
        a.item_name AS auction_item_name,
        b.max_price AS current_highest_bid
    FROM auction a
    LEFT OUTER JOIN (
        SELECT
            b1.auction,
            MAX(b1.price) max_price
        FROM bid b1
        GROUP BY b1.auction
    ) b ON a.id = b.auction
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q102 AS
    SELECT
        a.id AS auction_id,
        a.item_name AS auction_item_name,
        COUNT(b.auction) AS bid_count
    FROM auction a
    JOIN bid b ON a.id = b.auction
    GROUP BY a.id, a.item_name
    HAVING COUNT(b.auction) >= (
        SELECT COUNT(*) / COUNT(DISTINCT auction) FROM bid
    )
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q103 AS
    SELECT
        a.id AS auction_id,
        a.item_name AS auction_item_name
    FROM auction a
    WHERE a.id IN (
        SELECT b.auction FROM bid b
        GROUP BY b.auction
        HAVING COUNT(*) >= 20
    )
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q104 AS
    SELECT
        a.id AS auction_id,
        a.item_name AS auction_item_name
    FROM auction a
    WHERE a.id NOT IN (
        SELECT b.auction FROM bid b
        GROUP BY b.auction
        HAVING COUNT(*) < 20
    )
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q105 AS
    SELECT
        a.id AS auction_id,
        a.item_name AS auction_item_name,
        COUNT(b.auction) AS bid_count
    FROM auction a
    JOIN bid b ON a.id = b.auction
    GROUP BY a.id, a.item_name
    ORDER BY bid_count DESC
    LIMIT 1000
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q7_rewrite AS
    SELECT
      B.auction,
      B.price,
      B.bidder,
      B.date_time
    FROM (
      SELECT
        auction,
        price,
        bidder,
        date_time,
        /*use rank here to express top-N with ties*/
        rank() over (partition by window_end order by price desc) as price_rank
      FROM
        TUMBLE(bid, date_time, INTERVAL '10' SECOND)
    ) B
    WHERE price_rank <= 1
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');


CREATE SINK nexmark_q5_rewrite AS
    SELECT
      B.auction,
      B.num
    FROM (
      SELECT
          auction,
          num,
          /*use rank here to express top-N with ties*/
          rank() over (partition by starttime order by num desc) as num_rank
      FROM (
          SELECT bid.auction, count(*) AS num, window_start AS starttime
          FROM HOP(bid, date_time, INTERVAL '2' SECOND, INTERVAL '10' SECOND)
          GROUP BY window_start, bid.auction
      )
    ) B
    WHERE num_rank <= 1
    WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');
