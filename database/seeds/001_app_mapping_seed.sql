-- Initial application mapping seed. Extend after real data profiling.

INSERT INTO dim_app_mapping (raw_app_name, standard_app_name, app_category, invalid_app_flag)
VALUES
  ('Youtube_QUIC_Video', 'YouTube', 'long_video_ott', 0),
  ('TikTok_NonHTTP_Video', 'TikTok', 'short_video', 0),
  ('TikTok_Live', 'TikTok Live', 'live_video', 0),
  ('Instagram_NonHTTP_Video', 'Instagram Video', 'short_video', 0),
  ('Roblox', 'Roblox', 'game', 0),
  ('Roblox_Game', 'Roblox', 'game', 0),
  ('Steam_Game', 'Steam', 'game', 0),
  ('MobileLegends_Game', 'Mobile Legends', 'game', 0),
  ('Unity3D', 'Unity3D', 'invalid_app', 1),
  ('WII_Update', 'WII Update', 'invalid_app', 1),
  ('WII_Data', 'WII Data', 'invalid_app', 1)
ON DUPLICATE KEY UPDATE
  standard_app_name = VALUES(standard_app_name),
  app_category = VALUES(app_category),
  invalid_app_flag = VALUES(invalid_app_flag),
  updated_at = CURRENT_TIMESTAMP;
