import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { SideNavmenuComponent } from './side-navmenu.component';

describe('SideNavmenuComponent', () => {
  let component: SideNavmenuComponent;
  let fixture: ComponentFixture<SideNavmenuComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ SideNavmenuComponent ]
    })
    .compileComponents();
  }));

  beforeEach(() => {
    fixture = TestBed.createComponent(SideNavmenuComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
