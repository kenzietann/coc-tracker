import { Module } from '@nestjs/common';
import { ClashService } from './clash.service';
import { ClashController } from './clash.controller';
import { TypeOrmModule } from '@nestjs/typeorm';
import { VillageEntity } from 'src/entities/village.entity';

@Module({
  imports: [TypeOrmModule.forFeature([VillageEntity])],
  providers: [ClashService],
  controllers: [ClashController]
})
export class ClashModule {}
