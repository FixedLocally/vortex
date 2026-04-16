-- phpMyAdmin SQL Dump
-- version 5.2.1deb3
-- https://www.phpmyadmin.net/
--
-- Host: localhost:3306
-- Generation Time: Apr 16, 2026 at 10:52 PM
-- Server version: 10.11.8-MariaDB-0ubuntu0.24.04.1-log
-- PHP Version: 8.3.6

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";

--
-- Database: `sandwich_v2`
--

-- --------------------------------------------------------

--
-- Table structure for table `address_lookup_table`
--

CREATE TABLE `address_lookup_table` (
  `id` bigint(20) NOT NULL,
  `address` varchar(45) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- --------------------------------------------------------

--
-- Table structure for table `block_revenue`
--

CREATE TABLE `block_revenue` (
  `slot` bigint(20) NOT NULL,
  `fee` bigint(20) NOT NULL,
  `tips` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- --------------------------------------------------------

--
-- Table structure for table `leader_schedule`
--

CREATE TABLE `leader_schedule` (
  `slot` bigint(20) NOT NULL,
  `leader_id` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- --------------------------------------------------------

--
-- Table structure for table `vote_latencies`
--

CREATE TABLE `vote_latencies` (
  `vote_account` varchar(45) NOT NULL,
  `slot` bigint(20) NOT NULL,
  `latency` tinyint(4) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

--
-- Indexes for dumped tables
--

--
-- Indexes for table `address_lookup_table`
--
ALTER TABLE `address_lookup_table`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `leader` (`address`);

--
-- Indexes for table `block_revenue`
--
ALTER TABLE `block_revenue`
  ADD PRIMARY KEY (`slot`);

--
-- Indexes for table `leader_schedule`
--
ALTER TABLE `leader_schedule`
  ADD PRIMARY KEY (`slot`),
  ADD KEY `leader_id` (`leader_id`);

--
-- Indexes for table `vote_latencies`
--
ALTER TABLE `vote_latencies`
  ADD PRIMARY KEY (`vote_account`,`slot`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `address_lookup_table`
--
ALTER TABLE `address_lookup_table`
  MODIFY `id` bigint(20) NOT NULL AUTO_INCREMENT;

--
-- Constraints for dumped tables
--

--
-- Constraints for table `leader_schedule`
--
ALTER TABLE `leader_schedule`
  ADD CONSTRAINT `leader_schedule_ibfk_1` FOREIGN KEY (`leader_id`) REFERENCES `address_lookup_table` (`id`) ON DELETE NO ACTION ON UPDATE NO ACTION;
COMMIT;
