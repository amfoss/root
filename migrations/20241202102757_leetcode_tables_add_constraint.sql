ALTER TABLE leetcode_stats
ADD CONSTRAINT leetcode_stats_member_id_key UNIQUE (member_id);
ALTER TABLE codeforces_stats
ADD CONSTRAINT codeforces_stats_member_id_key UNIQUE (member_id);
