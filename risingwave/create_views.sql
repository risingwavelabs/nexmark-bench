CREATE VIEW bid
AS
SELECT (bid).auction, (bid).bidder, (bid).price, (bid).channel, (bid).url, (bid).extra, p_time, date_time FROM nexmark WHERE event_type = 2;

CREATE VIEW auction
AS
SELECT (auction).id, (auction).item_name, (auction).description, (auction).initial_bid, (auction).reserve, (auction).expires, (auction).seller, (auction).category, (auction).extra, p_time, date_time FROM nexmark WHERE event_type = 1;

CREATE VIEW person
AS
SELECT (person).id, (person).name, (person).email_address, (person).credit_card, (person).city, (person).state, (person).extra, p_time, date_time FROM nexmark WHERE event_type = 0;
