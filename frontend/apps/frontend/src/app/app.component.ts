import { Component } from '@angular/core';
import { TranslateService } from '@ngx-translate/core';

@Component({
  selector: 'frontend-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent {
  title = 'frontend';
  param = { value: 'world' };

  constructor(translate: TranslateService) {
    translate.setDefaultLang('en');

    translate.use('en');
  }
}
