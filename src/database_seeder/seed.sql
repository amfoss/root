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

-- Attendance (Original code - UNCHANGED)
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



-- StatusUpdateStreak (Original code - UNCHANGED)
INSERT INTO StatusUpdateStreak (
    member_id, current_streak, max_streak
)
SELECT 
    member_id,
    FLOOR(random() * 10 + 1)::INT,
    FLOOR(random() * 30 + 10)::INT
FROM member
ON CONFLICT (member_id) DO NOTHING;

-- Project (Original code - UNCHANGED)
INSERT INTO Project (member_id, title)
SELECT 
    m.member_id,
    CASE
        WHEN row_number() OVER (PARTITION BY m.member_id) % 3 = 0 THEN 'Machine Learning Project ' || m.member_id || '_' || row_number() OVER (PARTITION BY m.member_id)
        WHEN row_number() OVER (PARTITION BY m.member_id) % 3 = 1 THEN 'Web Development Project ' || m.member_id || '_' || row_number() OVER (PARTITION BY m.member_id)
        ELSE 'Data Analysis Project ' || m.member_id || '_' || row_number() OVER (PARTITION BY m.member_id)
    END
FROM member m
CROSS JOIN generate_series(1, 3) AS i  -- Create up to 3 projects per member
WHERE NOT EXISTS (
    SELECT 1 FROM Project p 
    WHERE p.member_id = m.member_id 
    AND p.title LIKE '%Project ' || m.member_id || '_%'
);

-- StatusUpdateHistory (Original code - UNCHANGED)
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


INSERT INTO member (
    roll_no, name, email, sex, year, hostel, mac_address, discord_id, group_id
) VALUES
    ('R001', 'Rihaan B H', 'rihaan@example.com', 'M', 3, 'Hostel A', '00:14:22:01:01:01', 'rihaan_discord', 1),
    ('R002', 'Abhinav M', 'abhinav@example.com', 'M', 3, 'Hostel B', '00:14:22:01:01:02', 'abhinav_discord', 1),
    ('R003', 'Shrivaths S Nair', 'shrivaths@example.com', 'M', 3, 'Hostel C', '00:14:22:01:01:03', 'shrivaths_discord', 2),
    ('R004', 'Hridesh MG', 'hridesh@example.com', 'M', 3, 'Hostel D', '00:14:22:01:01:04', 'hridesh_discord', 2),
    ('R005', 'Manas Varma K', 'manas@example.com', 'M', 3, 'Hostel E', '00:14:22:01:01:05', 'manas_discord', 3),
    ('R006', 'Chinmay Ajith', 'chinmay@example.com', 'M', 3, 'Hostel F', '00:14:22:01:01:06', 'chinmay_discord', 3),
    ('R008', 'Shravya K Suresh', 'shravya@example.com', 'F', 3, 'Hostel H', '00:14:22:01:01:08', 'shravya_discord', 4),
    ('R009', 'Swayam Agrahari', 'swayam@example.com', 'M', 3, 'Hostel I', '00:14:22:01:01:09', 'swayam_discord', 5),
    ('R010', 'Anamika V Menon', 'anamika@example.com', 'F', 3, 'Hostel J', '00:14:22:01:01:10', 'anamika_discord', 5)
ON CONFLICT (roll_no) DO NOTHING;

-- LeetCode statistics for specific members
WITH leetcode_members AS (
    SELECT 
        m.member_id,
        m.name,
        CASE 
            WHEN m.name = 'Rihaan B H' THEN 'rihaan1810'
            WHEN m.name = 'Abhinav M' THEN 'abhinavmohandas'
            WHEN m.name = 'Shrivaths S Nair' THEN 'Jatayu_2005'
            WHEN m.name = 'Hridesh MG' THEN 'hrideshmg'
            WHEN m.name = 'Shravya K Suresh' THEN 'shraavv'
            WHEN m.name = 'Swayam Agrahari' THEN 'swayam-agrahari'
            WHEN m.name = 'Anamika V Menon' THEN 'anamika_12'
            WHEN m.name = 'Souri S' THEN 'souri008_s'
            WHEN m.name = 'Keerthan K K' THEN 'keerthankk'
            WHEN m.name = 'Dheeraj M' THEN 'CrownDestro'
        END AS leetcode_username
    FROM member m
    WHERE m.name IN (
        'Rihaan B H', 'Abhinav M', 'Shrivaths S Nair', 'Hridesh MG', 
        'Shravya K Suresh', 'Swayam Agrahari', 'Anamika V Menon'
    )
)
INSERT INTO leetcode_stats (
    member_id, leetcode_username, problems_solved, easy_solved, 
    medium_solved, hard_solved, contests_participated, best_rank, total_contests
)
SELECT 
    member_id,
    leetcode_username,
    FLOOR(random() * 500 + 50)::INT,
    FLOOR(random() * 200 + 30)::INT,
    FLOOR(random() * 250 + 20)::INT,
    FLOOR(random() * 100 + 5)::INT,
    FLOOR(random() * 20 + 1)::INT,
    FLOOR(random() * 5000 + 100)::INT,
    FLOOR(random() * 30 + 5)::INT
FROM leetcode_members
WHERE leetcode_username IS NOT NULL
ON CONFLICT (member_id) DO NOTHING;

-- Codeforces statistics for specific members
WITH codeforces_members AS (
    SELECT 
        m.member_id,
        m.name,
        CASE 
            WHEN m.name = 'Atharva Unnikrishnan Nair' THEN 'atharva_04'
            WHEN m.name = 'Navaneeth' THEN 'navaneeth0041'
            WHEN m.name = 'Hridesh MG' THEN 'hrideshmg'
            WHEN m.name = 'Manas Varma K' THEN 'xX_Elektro_Xx'
            WHEN m.name = 'Chinmay Ajith' THEN 'chimnayyyy'
            WHEN m.name = 'Harikrishna TP' THEN 'harikrishna05'
            WHEN m.name = 'Vishnu Mohandas' THEN 'VishnuM_24'
            WHEN m.name = 'Mukund Menon' THEN 'CR1T1KAL16'
            WHEN m.name = 'G O Ashwin Praveen' THEN 'ashwinpraveengo'
            WHEN m.name = 'Aman V Shafeeq' THEN 'amansxcalibur'
            WHEN m.name = 'Gautham Mohanraj' THEN 'gauthammohanraj'
            WHEN m.name = 'Sabarinath J' THEN 'e_clipw_ze'
            WHEN m.name = 'Vishnu Tejas E' THEN 'he1senbrg'
        END AS codeforces_handle
    FROM member m
    WHERE m.name IN (
        'Hridesh MG', 'Manas Varma K', 'Chinmay Ajith', 'Harikrishna TP', 
        'Vishnu Mohandas', 'Mukund Menon', 'G O Ashwin Praveen', 'Aman V Shafeeq', 
        'Gautham Mohanraj', 'Sabarinath J', 'Vishnu Tejas E'
    )
)
INSERT INTO codeforces_stats (
    member_id, codeforces_handle, codeforces_rating, max_rating, contests_participated
)
SELECT 
    member_id,
    codeforces_handle,
    FLOOR(random() * 2000 + 800)::INT,
    FLOOR(random() * 500 + 1800)::INT,
    FLOOR(random() * 50 + 5)::INT
FROM codeforces_members
WHERE codeforces_handle IS NOT NULL
ON CONFLICT (member_id) DO NOTHING;

-- Leaderboard calculation (refactored for better readability)
INSERT INTO leaderboard (
    member_id, leetcode_score, codeforces_score, unified_score
)
SELECT 
    m.member_id,
    -- LeetCode score calculation
    COALESCE(ls.problems_solved * 2 + ls.contests_participated * 10, 0) AS leetcode_score,
    -- Codeforces score calculation with rating tiers
    COALESCE(
        CASE 
            WHEN cf.codeforces_rating < 1200 THEN (cf.codeforces_rating * 0.5 + cf.contests_participated * 5)::INT
            WHEN cf.codeforces_rating < 1600 THEN (cf.codeforces_rating * 0.7 + cf.contests_participated * 8)::INT
            WHEN cf.codeforces_rating < 1900 THEN (cf.codeforces_rating * 0.9 + cf.contests_participated * 12)::INT
            ELSE (cf.codeforces_rating * 1.1 + cf.contests_participated * 15)::INT
        END, 
        0
    ) AS codeforces_score,
    -- Combined unified score
    COALESCE(ls.problems_solved * 2 + ls.contests_participated * 10, 0) + 
    COALESCE(
        CASE 
            WHEN cf.codeforces_rating < 1200 THEN (cf.codeforces_rating * 0.5 + cf.contests_participated * 5)::INT
            WHEN cf.codeforces_rating < 1600 THEN (cf.codeforces_rating * 0.7 + cf.contests_participated * 8)::INT
            WHEN cf.codeforces_rating < 1900 THEN (cf.codeforces_rating * 0.9 + cf.contests_participated * 12)::INT
            ELSE (cf.codeforces_rating * 1.1 + cf.contests_participated * 15)::INT
        END, 
        0
    ) AS unified_score
FROM member m
LEFT JOIN leetcode_stats ls ON m.member_id = ls.member_id
LEFT JOIN codeforces_stats cf ON m.member_id = cf.member_id
ON CONFLICT (member_id) DO NOTHING;