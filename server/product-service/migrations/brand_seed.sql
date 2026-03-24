-- ============================================================
-- Brand seed data — extracted from product inventory
-- 97 brands, sorted alphabetically
-- ============================================================

TRUNCATE brand CASCADE;

INSERT INTO brand (name, code, tier, country) VALUES
    -- A
    ('Abercrombie & Fitch',     'ABF',  'premium', 'USA'),
    ('Adidas',                  'AD',   'premium', 'Germany'),
    ('Admiral',                 'ADM',  'mass',    'UK'),
    ('Arena',                   'ARN',  'mass',    'Italy'),
    ('Armani Jeans',            'AJ',   'premium', 'Italy'),
    ('Asics',                   'ASC',  'premium', 'Japan'),

    -- B
    ('Baldessarini',            'BLD',  'luxury',  'Germany'),
    ('Barbour',                 'BAR',  'premium', 'UK'),
    ('Birkenstock',             'BRK',  'premium', 'Germany'),
    ('Bogner',                  'BOG',  'luxury',  'Germany'),

    -- C
    ('Calvin Klein',            'CK',   'premium', 'USA'),
    ('Camel Active',            'CML',  'mass',    'Germany'),
    ('Champion',                'CHP',  'mass',    'USA'),
    ('Chanel',                  'CHN',  'luxury',  'France'),
    ('Clarks',                  'CLR',  'premium', 'UK'),
    ('Columbia',                'COL',  'premium', 'USA'),
    ('Converse',                'CNV',  'mass',    'USA'),
    ('Crocs',                   'CRC',  'mass',    'USA'),

    -- D
    ('Desigual',                'DSG',  'premium', 'Spain'),
    ('Diesel',                  'DSL',  'premium', 'Italy'),
    ('Disney',                  'DIS',  'mass',    'USA'),
    ('Drykorn',                 'DRK',  'premium', 'Germany'),
    ('Dsquared2',               'DS2',  'luxury',  'Italy'),

    -- E
    ('Ecco',                    'ECO',  'premium', 'Denmark'),
    ('Elkline',                 'ELK',  'mass',    'Germany'),
    ('Ellesse',                 'ELS',  'mass',    'Italy'),
    ('Escada',                  'ESC',  'luxury',  'Germany'),

    -- F
    ('Fjällräven',              'FJR',  'premium', 'Sweden'),
    ('FOX Racing',              'FOX',  'mass',    'USA'),

    -- G
    ('GANT',                    'GNT',  'premium', 'USA'),
    ('GAP',                     'GAP',  'mass',    'USA'),
    ('Geox',                    'GEX',  'premium', 'Italy'),
    ('Guess',                   'GES',  'premium', 'USA'),
    ('Guy Rover',               'GRV',  'luxury',  'Italy'),
    ('Gymshark',                'GYM',  'mass',    'UK'),

    -- H
    ('Helly Hansen',            'HH',   'premium', 'Norway'),
    ('Hollister',               'HOL',  'mass',    'USA'),
    ('Honey Punch',             'HNP',  'mass',    'USA'),
    ('Horsefeathers',           'HRS',  'mass',    'Czech Republic'),
    ('Hugo Boss',               'HB',   'luxury',  'Germany'),
    ('Hummel',                  'HML',  'mass',    'Denmark'),

    -- J
    ('J.Lindeberg',             'JLB',  'premium', 'Sweden'),
    ('Janet & Janet',           'JNJ',  'premium', 'Italy'),

    -- K
    ('Karl Lagerfeld',          'KL',   'luxury',  'Germany'),
    ('Keen',                    'KEE',  'premium', 'USA'),

    -- L
    ('La Martina',              'LMR',  'premium', 'Argentina'),
    ('Le Streghe',              'LST',  'premium', 'Italy'),
    ('Levi''s',                 'LEV',  'premium', 'USA'),
    ('Lloyd',                   'LLD',  'premium', 'Germany'),
    ('Louis Vuitton',           'LV',   'luxury',  'France'),
    ('Lululemon',               'LUL',  'premium', 'Canada'),

    -- M
    ('Marc Cain',               'MC',   'luxury',  'Germany'),
    ('Massimo Dutti',           'MD',   'premium', 'Spain'),
    ('Max Mara',                'MM',   'luxury',  'Italy'),
    ('McGregor',                'MGR',  'mass',    'Netherlands'),
    ('Merz b. Schwanen',        'MBS',  'luxury',  'Germany'),
    ('MET',                     'MET',  'premium', 'Italy'),
    ('More & More',             'MOR',  'mass',    'Germany'),
    ('Mustang',                 'MST',  'mass',    'Germany'),

    -- N
    ('Nakamura',                'NKM',  'mass',    'Austria'),
    ('Naketano',                'NKT',  'mass',    'Germany'),
    ('Napapijri',               'NAP',  'premium', 'Italy'),
    ('New Balance',             'NB',   'premium', 'USA'),
    ('Nike',                    'NK',   'premium', 'USA'),
    ('Nikkie',                  'NKK',  'premium', 'Netherlands'),

    -- O
    ('On Running',              'ON',   'premium', 'Switzerland'),
    ('O''Neill',                'ONL',  'mass',    'USA'),
    ('& Other Stories',         'AOS',  'premium', 'Sweden'),

    -- P
    ('Paul & Shark',            'PS',   'premium', 'Italy'),
    ('Peak Performance',        'PP',   'premium', 'Sweden'),
    ('Pedro del Hierro',        'PDH',  'premium', 'Spain'),

    -- R
    ('Ralph Lauren',            'RL',   'premium', 'USA'),
    ('Reebok',                  'RBK',  'mass',    'USA'),
    ('Replay',                  'RPL',  'premium', 'Italy'),
    ('Royal Robbins',           'RRB',  'mass',    'USA'),

    -- S
    ('Salewa',                  'SLW',  'premium', 'Italy'),
    ('Salomon',                 'SLM',  'premium', 'France'),
    ('Stella McCartney',        'SMC',  'luxury',  'UK'),
    ('Stohlquist',              'STQ',  'mass',    'USA'),
    ('Super Slam U.S.A.',       'SSU',  'mass',    'USA'),
    ('Superga',                 'SPG',  'mass',    'Italy'),
    ('Superdry',                'SD',   'mass',    'UK'),

    -- T
    ('The North Face',          'TNF',  'premium', 'USA'),
    ('Timberland',              'TMB',  'premium', 'USA'),
    ('Tommy Hilfiger',          'TH',   'premium', 'USA'),
    ('TradeMutt',               'TRM',  'mass',    'Australia'),
    ('Triumph',                 'TRI',  'mass',    'Germany'),
    ('2117 of Sweden',          '217',  'mass',    'Sweden'),

    -- U
    ('U.S. Polo Assn.',         'USP',  'mass',    'USA'),
    ('Under Armour',            'UA',   'premium', 'USA'),
    ('Uniqlo',                  'UNQ',  'mass',    'Japan'),

    -- V
    ('Vans',                    'VAN',  'mass',    'USA'),

    -- W
    ('Walbusch',                'WLB',  'mass',    'Germany'),
    ('Wrangler',                'WRG',  'mass',    'USA'),

    -- Y
    ('Yonex',                   'YNX',  'mass',    'Japan'),
    ('Yves Saint Laurent',      'YSL',  'luxury',  'France'),

    -- G-Star
    ('G-Star RAW',              'GSR',  'premium', 'Netherlands');