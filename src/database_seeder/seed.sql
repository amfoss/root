-- Member
INSERT INTO member (
    roll_no, name, email, sex, year, hostel, mac_address, discord_id, group_id
)
SELECT 
    'R' || LPAD(i::TEXT, 4, '0'),
    CASE 
        WHEN i % 5 = 0 THEN 'John Doe ' || i
        WHEN i % 5 = 1 THEN 'Jane Smith ' || i
        WHEN i % 5 = 2 THEN 'Alex Johnson ' || i
        WHEN i % 5 = 3 THEN 'Emily Davis ' || i
        ELSE 'Chris Brown ' || i
    END,
    CASE 
        WHEN i % 5 = 0 THEN 'john.doe' || i || '@example.com'
        WHEN i % 5 = 1 THEN 'jane.smith' || i || '@example.com'
        WHEN i % 5 = 2 THEN 'alex.johnson' || i || '@example.com'
        WHEN i % 5 = 3 THEN 'emily.davis' || i || '@example.com'
        ELSE 'chris.brown' || i || '@example.com'
    END,
    CASE 
        WHEN i % 2 = 0 THEN 'M'::sex_type 
        ELSE 'F'::sex_type 
    END,
    (i % 4) + 1,
    'Hostel ' || ((i % 5) + 1),
    '00:14:22:01:' || LPAD(TO_HEX(i), 2, '0') || ':' || LPAD(TO_HEX(i + 60), 2, '0'),
    'discord_user_' || i,
    (i % 8) + 1
FROM generate_series(1, 60) AS i
ON CONFLICT (roll_no) DO NOTHING;


-- Attendance
INSERT INTO Attendance (
    member_id, date, is_present, time_in, time_out
)
SELECT 
    m.member_id,
    CURRENT_DATE - ((i * 3) % 30),
    rnd.is_present,
    CASE WHEN rnd.is_present THEN rnd.time_in ELSE NULL END,
    CASE WHEN rnd.is_present THEN rnd.time_out ELSE NULL END
FROM generate_series(1, 600) AS i
JOIN (
    SELECT generate_series(1, 60) AS idx, member_id
    FROM member
) AS m ON (i % 60) + 1 = m.idx
JOIN (
    SELECT 
        TRUE AS is_present,
        '08:30'::TIME + (INTERVAL '1 minute' * (random() * 60)) AS time_in,
        '17:00'::TIME + (INTERVAL '1 minute' * (random() * 60)) AS time_out
    UNION ALL
    SELECT FALSE, NULL, NULL
) AS rnd ON TRUE
WHERE (random() < 0.75)
ON CONFLICT (member_id, date) DO NOTHING;


-- StatusUpdateHistory
INSERT INTO StatusUpdateHistory (
    member_id, date, is_updated
)
SELECT 
    m.member_id,
    CURRENT_DATE - ((i * 2) % 30),
    i % 2 = 0
FROM generate_series(1, 500) AS i
JOIN (
    SELECT generate_series(1, 60) AS idx, member_id
    FROM member
) AS m ON (i % 60) + 1 = m.idx
ON CONFLICT (member_id, date) DO NOTHING;
