use super::*;
impl TabletUi {
    pub(super) fn sheets(&mut self, app: &mut PhotocraftApp, ctx: &egui::Context) {
        let Some(sheet) = self.sheet else { return };
        let mut close = false;
        let title = match sheet {
            Sheet::Tools => "Tools",
            Sheet::Commands => "Commands",
            Sheet::Files => "Documents & files",
            Sheet::Brush => "Brush studio",
            Sheet::Channels => "Channels",
            Sheet::Paths => "Paths",
        };
        let width = (ctx.content_rect().width() - 40.).clamp(240., 640.);
        let modal=egui::Modal::new(egui::Id::new("ipad-sheet")).show(ctx,|ui|{
            ui.set_width(width);
            ui.spacing_mut().interact_size=vec2(44.,44.);
            ui.horizontal(|ui|{ui.heading(title);if button(ui,"Close",false).clicked(){close=true;}});
            if matches!(sheet,Sheet::Tools|Sheet::Commands){ui.add_sized([width,44.],egui::TextEdit::singleline(&mut self.search).hint_text("Search…"));}
            egui::ScrollArea::vertical().max_height((ctx.content_rect().height()-180.).max(120.)).show(ui,|ui|{
                match sheet{
                    Sheet::Brush => self.brush_studio(app,ui),
                    Sheet::Channels => self.channels(app,ui),
                    Sheet::Paths => self.paths(app,ui),
                    Sheet::Tools=>{
                        let query=self.search.to_lowercase();
                        let columns=(width/160.).floor().max(1.)as usize;
                        egui::Grid::new("ipad-tool-grid").spacing(vec2(8.,8.)).show(ui,|ui|{
                            let mut count=0;
                            for tool in Tool::ALL{
                                if !tool.label().to_lowercase().contains(&query){continue;}
                                if ui.add_sized([width/columns as f32-12.,64.],Button::new(tool.label()).selected(app.ui.tool==tool)).clicked(){app.ui.tool=tool;close=true;}
                                count+=1;if count%columns==0{ui.end_row();}
                            }
                        });
                    }
                    Sheet::Commands=>{
                        let items=menus::menu_items(app);
                        ui.horizontal_wrapped(|ui|{
                            for category in ["All","File","Edit","Image","Layer","Type","Select","Filter","View","Window"]{
                                let value=if category=="All"{""}else{category};
                                if button(ui,category,self.category==value).clicked(){self.category=value.into();}
                            }
                        });
                        let query=self.search.to_lowercase();
                        let mut count=0;
                        for item in items{
                            if item.label=="---" || item.id.is_empty(){continue;}
                            if !self.category.is_empty()&&item.path.first()!=Some(&self.category){continue;}
                            let path=item.path.join(" › ");
                            if !format!("{} {}",item.label,path).to_lowercase().contains(&query){continue;}
                            count+=1;
                            let label=format!("{}\n{}{}",item.label,path,if menus::is_live(&item.id){""}else{" · Not implemented upstream"});
                            if ui.add_enabled(item.enabled,Button::new(label).min_size(vec2(width-20.,58.))).clicked(){self.invoke(app,ctx,&item.id,json!({}));close=true;}
                        }
                        if count==0{ui.label("No matching commands.");}
                    }
                    Sheet::Files=>{
                        ui.horizontal_wrapped(|ui|{
                            for (label,id) in [("New image","file.new"),("Open…","file.open"),("Save","file.save"),("Save as…","file.saveAs"),("Export…","file.export.exportAs"),("PNG","file.export.quickExportAsPng")]{
                                if ui.add_enabled(menus::is_enabled(app,id),Button::new(label).min_size(vec2(120.,48.))).clicked(){self.invoke(app,ctx,id,json!({}));close=true;}
                            }
                        });
                        ui.separator();ui.strong("Open documents");
                        let active=app.session.active_index();
                        let docs:Vec<_>=app.session.documents().iter().map(|d|format!("{}{}",d.doc.name,if d.is_dirty(){" •"}else{""})).collect();
                        for (i,name) in docs.iter().enumerate(){if full_button(ui,name,active==Some(i)).clicked(){app.session.set_active(i);close=true;}}
                        ui.add_space(12.);ui.label("Save downloads to Files on iPad. Save before closing or reloading this page.");
                    }
                }
            });
        });
        if close || modal.should_close() {
            self.sheet = None;
        }
    }
}
