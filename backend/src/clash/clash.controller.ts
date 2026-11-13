import { BadRequestException, Body, Controller, Post, Get, Patch, Param, ParseUUIDPipe } from '@nestjs/common';
import { ClashService } from './clash.service';

@Controller('clash')
export class ClashController {
  constructor(private readonly clashService: ClashService) {}

  @Post('import')
  importVillage(@Body() body: unknown) {
    return this.clashService.importVillage(body);
  }

  @Get('villages')
  getVillages(){
    return this.clashService.getVillages();
  }

  @Patch('villages/:id')
  updateVillage(@Param('id', ParseUUIDPipe) id: string, @Body() body: unknown) {
    if (typeof body !== 'object' || body === null || Array.isArray(body)) {
      throw new BadRequestException('Request body must be an object');
    }
    if (!('label' in body)) {
      throw new BadRequestException('Request body must contain a label');
    }
    return this.clashService.updateVillageLabel(id, body.label);
  }
}
