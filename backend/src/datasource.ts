import 'dotenv/config';
import { DataSource } from "typeorm";

export const AppDataSource = new DataSource({
  type:         'mariadb',
  database:     process.env.DATABASE_NAME,
  host:         'localhost',
  username:     process.env.DATABASE_USER,
  password:     process.env.DATABASE_PASSWORD,
  port:         Number(process.env.DATABASE_PORT),
  entities:     [__dirname + '/entities/*.entity{.ts,.js}'],
  migrations:   [__dirname + '/../migrations/*{.ts,.js}'],
  logging:      true,
  synchronize:  false
})