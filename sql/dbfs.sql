SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;


CREATE TABLE `block` (
  `inode_id` int(10) UNSIGNED NOT NULL,
  `block_id` int(10) UNSIGNED NOT NULL,
  `data` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `file` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) NOT NULL,
  `inode_id` int(10) UNSIGNED NOT NULL,
  `parent_id` int(10) UNSIGNED DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `file_types` (
  `id` char(1) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL,
  `description` varchar(50) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

INSERT INTO `file_types` (`id`, `description`) VALUES
('-', 'Regular file'),
('d', 'Directory'),
('l', 'Symbolic link');

CREATE TABLE `group` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `inode` (
  `id` int(10) UNSIGNED NOT NULL,
  `owner` int(10) UNSIGNED NOT NULL,
  `group` int(10) UNSIGNED NOT NULL,
  `file_type` char(1) NOT NULL,
  `special_bits` tinyint(4) NOT NULL DEFAULT 0,
  `user_perm` tinyint(4) NOT NULL DEFAULT 0,
  `group_perm` tinyint(4) NOT NULL DEFAULT 0,
  `other_perm` tinyint(4) NOT NULL DEFAULT 0,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `modified_at` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  `accessed_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `permissions` (
  `id` tinyint(4) NOT NULL,
  `can_read` tinyint(1) NOT NULL,
  `can_write` tinyint(1) NOT NULL,
  `can_execute` tinyint(1) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

INSERT INTO `permissions` (`id`, `can_read`, `can_write`, `can_execute`) VALUES
(0, 0, 0, 0),
(1, 0, 0, 1),
(2, 0, 1, 0),
(3, 0, 1, 1),
(4, 1, 0, 0),
(5, 1, 0, 1),
(6, 1, 1, 0),
(7, 1, 1, 1);

CREATE TABLE `special_bits` (
  `id` tinyint(4) NOT NULL,
  `setuid` tinyint(1) NOT NULL,
  `setgid` tinyint(1) NOT NULL,
  `sticky` tinyint(1) NOT NULL,
  `description` varchar(100) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

INSERT INTO `special_bits` (`id`, `setuid`, `setgid`, `sticky`, `description`) VALUES
(0, 0, 0, 0, 'No special bits'),
(1, 0, 0, 1, 'Sticky bit'),
(2, 0, 1, 0, 'Set group ID'),
(3, 0, 1, 1, 'Set group ID and Sticky bit'),
(4, 1, 0, 0, 'Set user ID'),
(5, 1, 0, 1, 'Set user ID and Sticky bit'),
(6, 1, 1, 0, 'Set user ID and Set group ID'),
(7, 1, 1, 1, 'Set user ID, Set group ID, and Sticky bit');

CREATE TABLE `user` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

ALTER TABLE `block`
  ADD PRIMARY KEY (`inode_id`,`block_id`);

ALTER TABLE `file`
  ADD PRIMARY KEY (`id`),
  ADD KEY `file_inode` (`inode_id`),
  ADD KEY `file_parent` (`parent_id`);

ALTER TABLE `file_types`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `group`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `inode`
  ADD PRIMARY KEY (`id`),
  ADD KEY `inode_owner` (`owner`),
  ADD KEY `inode_group` (`group`),
  ADD KEY `file_type` (`file_type`),
  ADD KEY `special_bits` (`special_bits`),
  ADD KEY `user_perm` (`user_perm`),
  ADD KEY `group_perm` (`group_perm`),
  ADD KEY `other_perm` (`other_perm`);

ALTER TABLE `permissions`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `special_bits`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `user`
  ADD PRIMARY KEY (`id`);


ALTER TABLE `file`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=1;

ALTER TABLE `group`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=1;

ALTER TABLE `inode`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=1;

ALTER TABLE `user`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=1;


ALTER TABLE `block`
  ADD CONSTRAINT `block_inode` FOREIGN KEY (`inode_id`) REFERENCES `inode` (`id`);

ALTER TABLE `file`
  ADD CONSTRAINT `file_inode` FOREIGN KEY (`inode_id`) REFERENCES `inode` (`id`),
  ADD CONSTRAINT `file_parent` FOREIGN KEY (`parent_id`) REFERENCES `file` (`id`);

ALTER TABLE `inode`
  ADD CONSTRAINT `inode_file_type` FOREIGN KEY (`file_type`) REFERENCES `file_types` (`id`),
  ADD CONSTRAINT `inode_group` FOREIGN KEY (`group`) REFERENCES `group` (`id`),
  ADD CONSTRAINT `inode_group_perm` FOREIGN KEY (`group_perm`) REFERENCES `permissions` (`id`),
  ADD CONSTRAINT `inode_other_perm` FOREIGN KEY (`other_perm`) REFERENCES `permissions` (`id`),
  ADD CONSTRAINT `inode_owner` FOREIGN KEY (`owner`) REFERENCES `user` (`id`),
  ADD CONSTRAINT `inode_special_bits` FOREIGN KEY (`special_bits`) REFERENCES `special_bits` (`id`),
  ADD CONSTRAINT `inode_user_perm` FOREIGN KEY (`user_perm`) REFERENCES `permissions` (`id`);
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
