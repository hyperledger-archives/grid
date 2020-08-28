-- Copyright 2019 Cargill Incorporated
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.
-- ------------------------------------------------------------------------------

--- This view queries the database and joins a reporter with the correspondent
--- metadata from the agent table.
CREATE VIEW reporter_to_agent_metadata
AS
  SELECT id,
         property_name,
         record_id,
         public_key,
         authorized,
         reporter_index,
         metadata,
         reporter_end_block_num
  FROM   (SELECT Row_number()
                   OVER (
                     partition BY id
                     ORDER BY agent_end_block_num) AS RowNum,
                 *
          FROM   (SELECT reporter.id,
                         reporter.property_name,
                         reporter.record_id,
                         reporter.reporter_index,
                         reporter.authorized,
                         reporter.public_key,
                         reporter.end_block_num AS "reporter_end_block_num",
                         agent.end_block_num    AS "agent_end_block_num",
                         agent.metadata
                  FROM   reporter
                         LEFT JOIN agent
                                ON reporter.public_key = agent.public_key
                                   AND reporter.end_block_num <=
                                       agent.end_block_num) AS
                 join_tables) X
  WHERE  rownum = 1;


--- This view queries the database and joins a reported_value with the
--- correspondent reporter with the metadata from the agent table.
CREATE VIEW reported_value_reporter_to_agent_metadata
AS
  SELECT id,
         property_name,
         record_id,
         reporter_index,
         timestamp,
         data_type,
         bytes_value,
         boolean_value,
         number_value,
         string_value,
         enum_value,
         struct_values,
         lat_long_value,
         public_key,
         authorized,
         metadata,
         reported_value_end_block_num,
         reporter_end_block_num
  FROM   (SELECT Row_number()
                   OVER (
                     partition BY id
                     ORDER BY reporter_end_block_num) AS RowNum,
                 *
          FROM   (SELECT reported_value.id,
                         reported_value.property_name,
                         reported_value.record_id,
                         reported_value.reporter_index,
                         reported_value.timestamp,
                         reported_value.data_type,
                         reported_value.bytes_value,
                         reported_value.boolean_value,
                         reported_value.number_value,
                         reported_value.string_value,
                         reported_value.enum_value,
                         reported_value.struct_values,
                         reported_value.lat_long_value,
                         reported_value.end_block_num AS
                         "reported_value_end_block_num",
                         reporter_to_agent_metadata.reporter_end_block_num,
                         reporter_to_agent_metadata.public_key,
                         reporter_to_agent_metadata.authorized,
                         reporter_to_agent_metadata.metadata
                  FROM   reported_value
                         LEFT JOIN reporter_to_agent_metadata
                                ON reported_value.record_id =
                                   reporter_to_agent_metadata.record_id
                                   AND reported_value.property_name =
                                       reporter_to_agent_metadata.property_name
                                   AND reported_value.reporter_index =
                                       reporter_to_agent_metadata.reporter_index
                                   AND reported_value.end_block_num <=
  reporter_to_agent_metadata.reporter_end_block_num) AS
  join_tables) X
  WHERE  rownum = 1;
