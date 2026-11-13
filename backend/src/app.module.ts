import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { ClashModule } from './clash/clash.module';
import { TypeOrmModule } from '@nestjs/typeorm';
import { AppDataSource } from './datasource';

@Module({
  imports: [ClashModule, TypeOrmModule.forRoot(AppDataSource.options)],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
