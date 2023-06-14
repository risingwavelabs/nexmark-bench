CREATE VIEW bid 
AS
SELECT (bid).* FROM nexmark WHERE event_type = 2;

CREATE VIEW auction 
AS
SELECT (auction).* FROM nexmark WHERE event_type = 1;

CREATE VIEW person 
AS
SELECT (person).* FROM nexmark WHERE event_type = 0;
